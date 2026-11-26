/*
** EPITECH PROJECT, 2026
** EpiQuest
** File description:
** importer.rs
*/

use crate::error::Result;
use crate::model::{Class, Item, Monster, Snapshot, Species, Spell};
use crate::snapshot::sha256_hex;
use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::BTreeMap;

const IMPORT_LOCK_ID: i64 = 0x5352_4449_4d50;

#[derive(Debug, Clone, Copy, Default)]
pub struct ImportOptions {
    pub force: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct KindReport {
    pub total: usize,
    pub inserted: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub stale: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportOutcome {
    UpToDate {
        checksum: String,
    },
    Imported {
        reports: BTreeMap<String, KindReport>,
    },
    DryRun {
        reports: BTreeMap<String, KindReport>,
    },
}

fn row_hash<T: Serialize>(entry: &T) -> String {
    sha256_hex(&serde_json::to_vec(entry).expect("SRD entries are always serializable"))
}

fn count(report: &mut KindReport, outcome: Option<bool>) {
    report.total += 1;
    match outcome {
        Some(true) => report.inserted += 1,
        Some(false) => report.updated += 1,
        None => report.unchanged += 1,
    }
}

async fn stale(
    tx: &mut Transaction<'_, Postgres>,
    sql: &'static str,
    document: &str,
    slugs: Vec<String>,
) -> Result<i64> {
    Ok(sqlx::query_scalar(sql)
        .bind(document)
        .bind(slugs)
        .fetch_one(&mut **tx)
        .await?)
}

async fn upsert_classes(
    tx: &mut Transaction<'_, Postgres>,
    doc: &str,
    rows: &[Class],
) -> Result<KindReport> {
    let mut report = KindReport::default();

    for row in rows {
        let outcome: Option<bool> = sqlx::query_scalar(
            r#"
            INSERT INTO classes (slug, name, hit_die, data, source_document, content_hash)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (slug) DO UPDATE SET
                name = EXCLUDED.name, hit_die = EXCLUDED.hit_die, data = EXCLUDED.data,
                source_document = EXCLUDED.source_document,
                content_hash = EXCLUDED.content_hash, updated_at = NOW()
            WHERE classes.content_hash IS DISTINCT FROM EXCLUDED.content_hash
            RETURNING (xmax = 0)
            "#,
        )
        .bind(&row.slug)
        .bind(&row.name)
        .bind(row.hit_die)
        .bind(&row.data)
        .bind(doc)
        .bind(row_hash(row))
        .fetch_optional(&mut **tx)
        .await?;
        count(&mut report, outcome);
    }
    report.stale = stale(
        tx,
        "SELECT COUNT(*) FROM classes WHERE source_document = $1 AND slug <> ALL($2)",
        doc,
        rows.iter().map(|r| r.slug.clone()).collect(),
    )
    .await?;
    Ok(report)
}

async fn upsert_species(
    tx: &mut Transaction<'_, Postgres>,
    doc: &str,
    rows: &[Species],
) -> Result<KindReport> {
    let mut report = KindReport::default();

    for row in rows {
        let outcome: Option<bool> = sqlx::query_scalar(
            r#"
            INSERT INTO species (slug, name, speed, size, data, source_document, content_hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (slug) DO UPDATE SET
                name = EXCLUDED.name, speed = EXCLUDED.speed, size = EXCLUDED.size,
                data = EXCLUDED.data, source_document = EXCLUDED.source_document,
                content_hash = EXCLUDED.content_hash, updated_at = NOW()
            WHERE species.content_hash IS DISTINCT FROM EXCLUDED.content_hash
            RETURNING (xmax = 0)
            "#,
        )
        .bind(&row.slug)
        .bind(&row.name)
        .bind(row.speed)
        .bind(&row.size)
        .bind(&row.data)
        .bind(doc)
        .bind(row_hash(row))
        .fetch_optional(&mut **tx)
        .await?;
        count(&mut report, outcome);
    }
    report.stale = stale(
        tx,
        "SELECT COUNT(*) FROM species WHERE source_document = $1 AND slug <> ALL($2)",
        doc,
        rows.iter().map(|r| r.slug.clone()).collect(),
    )
    .await?;
    Ok(report)
}

async fn upsert_spells(
    tx: &mut Transaction<'_, Postgres>,
    doc: &str,
    rows: &[Spell],
) -> Result<KindReport> {
    let mut report = KindReport::default();

    for row in rows {
        let outcome: Option<bool> = sqlx::query_scalar(
            r#"
            INSERT INTO spells (slug, name, level, school, casting_time, range, duration,
                                components, data, source_document, content_hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (slug) DO UPDATE SET
                name = EXCLUDED.name, level = EXCLUDED.level, school = EXCLUDED.school,
                casting_time = EXCLUDED.casting_time, range = EXCLUDED.range,
                duration = EXCLUDED.duration, components = EXCLUDED.components,
                data = EXCLUDED.data, source_document = EXCLUDED.source_document,
                content_hash = EXCLUDED.content_hash, updated_at = NOW()
            WHERE spells.content_hash IS DISTINCT FROM EXCLUDED.content_hash
            RETURNING (xmax = 0)
            "#,
        )
        .bind(&row.slug)
        .bind(&row.name)
        .bind(row.level)
        .bind(&row.school)
        .bind(&row.casting_time)
        .bind(&row.range)
        .bind(&row.duration)
        .bind(&row.components)
        .bind(&row.data)
        .bind(doc)
        .bind(row_hash(row))
        .fetch_optional(&mut **tx)
        .await?;
        count(&mut report, outcome);
    }
    report.stale = stale(
        tx,
        "SELECT COUNT(*) FROM spells WHERE source_document = $1 AND slug <> ALL($2)",
        doc,
        rows.iter().map(|r| r.slug.clone()).collect(),
    )
    .await?;
    Ok(report)
}

async fn upsert_items(
    tx: &mut Transaction<'_, Postgres>,
    doc: &str,
    rows: &[Item],
) -> Result<KindReport> {
    let mut report = KindReport::default();

    for row in rows {
        let outcome: Option<bool> = sqlx::query_scalar(
            r#"
            INSERT INTO items (slug, name, item_type, rarity, data, source_document, content_hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (slug) DO UPDATE SET
                name = EXCLUDED.name, item_type = EXCLUDED.item_type,
                rarity = EXCLUDED.rarity, data = EXCLUDED.data,
                source_document = EXCLUDED.source_document,
                content_hash = EXCLUDED.content_hash, updated_at = NOW()
            WHERE items.content_hash IS DISTINCT FROM EXCLUDED.content_hash
            RETURNING (xmax = 0)
            "#,
        )
        .bind(&row.slug)
        .bind(&row.name)
        .bind(&row.item_type)
        .bind(&row.rarity)
        .bind(&row.data)
        .bind(doc)
        .bind(row_hash(row))
        .fetch_optional(&mut **tx)
        .await?;
        count(&mut report, outcome);
    }
    report.stale = stale(
        tx,
        "SELECT COUNT(*) FROM items WHERE source_document = $1 AND slug <> ALL($2)",
        doc,
        rows.iter().map(|r| r.slug.clone()).collect(),
    )
    .await?;
    Ok(report)
}

async fn upsert_monsters(
    tx: &mut Transaction<'_, Postgres>,
    doc: &str,
    rows: &[Monster],
) -> Result<KindReport> {
    let mut report = KindReport::default();

    for row in rows {
        let outcome: Option<bool> = sqlx::query_scalar(
            r#"
            INSERT INTO monsters (slug, name, challenge_rating, size, monster_type, data,
                                  source_document, content_hash)
            VALUES ($1, $2, $3::float8::numeric, $4, $5, $6, $7, $8)
            ON CONFLICT (slug) DO UPDATE SET
                name = EXCLUDED.name, challenge_rating = EXCLUDED.challenge_rating,
                size = EXCLUDED.size, monster_type = EXCLUDED.monster_type,
                data = EXCLUDED.data, source_document = EXCLUDED.source_document,
                content_hash = EXCLUDED.content_hash, updated_at = NOW()
            WHERE monsters.content_hash IS DISTINCT FROM EXCLUDED.content_hash
            RETURNING (xmax = 0)
            "#,
        )
        .bind(&row.slug)
        .bind(&row.name)
        .bind(row.challenge_rating)
        .bind(&row.size)
        .bind(&row.monster_type)
        .bind(&row.data)
        .bind(doc)
        .bind(row_hash(row))
        .fetch_optional(&mut **tx)
        .await?;
        count(&mut report, outcome);
    }
    report.stale = stale(
        tx,
        "SELECT COUNT(*) FROM monsters WHERE source_document = $1 AND slug <> ALL($2)",
        doc,
        rows.iter().map(|r| r.slug.clone()).collect(),
    )
    .await?;
    Ok(report)
}

pub async fn last_checksum(pool: &PgPool, document: &str) -> Result<Option<String>> {
    Ok(sqlx::query_scalar(
        "SELECT checksum FROM srd_imports WHERE document = $1 ORDER BY imported_at DESC LIMIT 1",
    )
    .bind(document)
    .fetch_optional(pool)
    .await?)
}

pub async fn import(
    pool: &PgPool,
    snapshot: &Snapshot,
    options: ImportOptions,
) -> Result<ImportOutcome> {
    let manifest = &snapshot.manifest;
    let doc = manifest.document.as_str();
    let mut tx = pool.begin().await?;

    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(IMPORT_LOCK_ID)
        .execute(&mut *tx)
        .await?;
    let last: Option<String> = sqlx::query_scalar(
        "SELECT checksum FROM srd_imports WHERE document = $1 ORDER BY imported_at DESC LIMIT 1",
    )
    .bind(doc)
    .fetch_optional(&mut *tx)
    .await?;

    if !options.force && last.as_deref() == Some(manifest.checksum.as_str()) {
        tx.rollback().await?;
        return Ok(ImportOutcome::UpToDate {
            checksum: manifest.checksum.clone(),
        });
    }
    let mut reports = BTreeMap::new();

    reports.insert(
        "classes".into(),
        upsert_classes(&mut tx, doc, &snapshot.classes).await?,
    );
    reports.insert(
        "species".into(),
        upsert_species(&mut tx, doc, &snapshot.species).await?,
    );
    reports.insert(
        "spells".into(),
        upsert_spells(&mut tx, doc, &snapshot.spells).await?,
    );
    reports.insert(
        "items".into(),
        upsert_items(&mut tx, doc, &snapshot.items).await?,
    );
    reports.insert(
        "monsters".into(),
        upsert_monsters(&mut tx, doc, &snapshot.monsters).await?,
    );
    if options.dry_run {
        tx.rollback().await?;
        return Ok(ImportOutcome::DryRun { reports });
    }
    sqlx::query(
        r#"
        INSERT INTO srd_imports (document, format_version, checksum, source, counts)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(doc)
    .bind(manifest.format_version as i32)
    .bind(&manifest.checksum)
    .bind(&manifest.source)
    .bind(serde_json::to_value(&reports).expect("reports are serializable"))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(ImportOutcome::Imported { reports })
}
