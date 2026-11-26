/*
** EPITECH PROJECT, 2026
** EpiQuest
** File description:
** import.rs
*/

use serde_json::json;
use sqlx::{PgPool, postgres::PgPoolOptions};
use srd::model::{Class, Item, Monster, Species, Spell};
use srd::{FORMAT_VERSION, ImportOptions, ImportOutcome, Manifest, Snapshot, snapshot};
use std::collections::BTreeMap;

async fn setup_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://app:app@localhost:5432/app".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Failed to connect to test database");

    srd::MIGRATOR
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    pool
}

fn fixture() -> Snapshot {
    let id = uuid::Uuid::new_v4().simple().to_string();
    let s = |name: &str| format!("t{}-{name}", &id[..12]);

    Snapshot {
        manifest: Manifest {
            format_version: FORMAT_VERSION,
            document: s("doc"),
            document_name: "Test fixture".into(),
            source: "manual".into(),
            source_url: None,
            license: "CC-BY-4.0".into(),
            generated_at: "2026-01-01T00:00:00Z".into(),
            files: BTreeMap::new(),
            checksum: String::new(),
        },
        classes: vec![Class {
            slug: s("fighter"),
            name: "Fighter".into(),
            hit_die: 10,
            data: json!({"subclasses": []}),
        }],
        species: vec![Species {
            slug: s("human"),
            name: "Human".into(),
            speed: 30,
            size: "Medium".into(),
            data: json!({}),
        }],
        spells: vec![Spell {
            slug: s("fireball"),
            name: "Fireball".into(),
            level: 3,
            school: "Evocation".into(),
            casting_time: "action".into(),
            range: "150 feet".into(),
            duration: "instantaneous".into(),
            components: "V, S, M".into(),
            data: json!({"damage_roll": "8d6"}),
        }],
        items: vec![Item {
            slug: s("rope"),
            name: "Rope".into(),
            item_type: "Adventuring Gear".into(),
            rarity: None,
            data: json!({"magic": false}),
        }],
        monsters: vec![Monster {
            slug: s("rat"),
            name: "Rat".into(),
            challenge_rating: 0.125,
            size: "Tiny".into(),
            monster_type: "Beast".into(),
            data: json!({}),
        }],
    }
}

fn round_trip(snap: &Snapshot) -> Snapshot {
    let dir = std::env::temp_dir().join(format!("srd-test-{}", uuid::Uuid::new_v4()));
    snapshot::write(snap, &dir).unwrap();
    let read = snapshot::read(&dir).unwrap();

    std::fs::remove_dir_all(&dir).ok();
    read
}

fn reports(outcome: ImportOutcome) -> BTreeMap<String, srd::KindReport> {
    match outcome {
        ImportOutcome::Imported { reports } | ImportOutcome::DryRun { reports } => reports,
        ImportOutcome::UpToDate { .. } => panic!("expected an import, got UpToDate"),
    }
}

#[test]
fn snapshot_round_trip_is_deterministic() {
    let snap = fixture();
    let a = round_trip(&snap);
    let b = round_trip(&snap);

    assert_eq!(a.manifest.checksum, b.manifest.checksum);
    assert_eq!(a.spells, snap.spells);
}

#[test]
fn tampered_file_is_rejected() {
    let dir = std::env::temp_dir().join(format!("srd-test-{}", uuid::Uuid::new_v4()));

    snapshot::write(&fixture(), &dir).unwrap();
    std::fs::write(dir.join("spells.json"), "[]\n").unwrap();
    assert!(matches!(
        snapshot::read(&dir),
        Err(srd::SrdError::ChecksumMismatch { .. })
    ));
    snapshot::rehash(&dir).unwrap();
    assert!(snapshot::read(&dir).unwrap().spells.is_empty());
    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn import_is_replayable() {
    let pool = setup_pool().await;
    let snap = round_trip(&fixture());

    let first = reports(
        srd::import(&pool, &snap, ImportOptions::default())
            .await
            .unwrap(),
    );
    assert!(first.values().all(|r| r.inserted == 1 && r.updated == 0));

    let second = srd::import(&pool, &snap, ImportOptions::default())
        .await
        .unwrap();
    assert!(matches!(second, ImportOutcome::UpToDate { .. }));

    let force = ImportOptions {
        force: true,
        ..Default::default()
    };
    let forced = reports(srd::import(&pool, &snap, force).await.unwrap());
    assert!(forced.values().all(|r| r.unchanged == 1 && r.inserted == 0));

    let cr: f64 =
        sqlx::query_scalar("SELECT challenge_rating::float8 FROM monsters WHERE slug = $1")
            .bind(&snap.monsters[0].slug)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cr, 0.125);
}

#[tokio::test]
async fn changed_entry_is_updated_and_keeps_its_id() {
    let pool = setup_pool().await;
    let mut snap = round_trip(&fixture());

    srd::import(&pool, &snap, ImportOptions::default())
        .await
        .unwrap();
    let id_before: uuid::Uuid = sqlx::query_scalar("SELECT id FROM spells WHERE slug = $1")
        .bind(&snap.spells[0].slug)
        .fetch_one(&pool)
        .await
        .unwrap();

    snap.spells[0].data = json!({"damage_roll": "9d6"});
    let snap = round_trip(&snap);
    let r = reports(
        srd::import(&pool, &snap, ImportOptions::default())
            .await
            .unwrap(),
    );

    assert_eq!(r["spells"].updated, 1);
    assert_eq!(r["monsters"].unchanged, 1);
    let id_after: uuid::Uuid = sqlx::query_scalar("SELECT id FROM spells WHERE slug = $1")
        .bind(&snap.spells[0].slug)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(id_before, id_after);
}

#[tokio::test]
async fn dry_run_writes_nothing() {
    let pool = setup_pool().await;
    let snap = round_trip(&fixture());
    let dry = ImportOptions {
        dry_run: true,
        ..Default::default()
    };

    reports(srd::import(&pool, &snap, dry).await.unwrap());
    let last = srd::importer::last_checksum(&pool, &snap.manifest.document)
        .await
        .unwrap();
    assert!(last.is_none());
}

#[tokio::test]
async fn removed_entry_is_reported_stale() {
    let pool = setup_pool().await;
    let mut snap = round_trip(&fixture());

    srd::import(&pool, &snap, ImportOptions::default())
        .await
        .unwrap();
    snap.items.clear();
    let snap = round_trip(&snap);
    let r = reports(
        srd::import(&pool, &snap, ImportOptions::default())
            .await
            .unwrap(),
    );

    assert_eq!(r["items"].stale, 1);
}
