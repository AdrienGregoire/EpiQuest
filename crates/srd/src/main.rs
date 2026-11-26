/*
** EPITECH PROJECT, 2026
** EpiQuest
** File description:
** main.rs
*/

use sqlx::postgres::PgPoolOptions;
use srd::open5e::{DEFAULT_BASE_URL, Open5eClient};
use srd::{ImportOptions, ImportOutcome, KindReport, snapshot};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

const DEFAULT_DOCUMENT: &str = "srd-2024";
const DEFAULT_ROOT: &str = "data/srd";

const USAGE: &str = "\
Usage: srd-import <command> [options]

Commands:
  fetch    Download from Open5e and write a snapshot      (network)
  verify   Check snapshot checksums and content            (offline)
  import   Import the snapshot into PostgreSQL             (offline)
  rehash   Recompute manifest after hand-editing SRD files (offline)
  status   Show last imported checksum vs snapshot         (offline)

Options:
  --document <key>   SRD document key        [default: srd-2024]
  --dir <path>       Snapshot directory      [default: data/srd/<document>]
  --base-url <url>   Open5e API base URL     [default: https://api.open5e.com/v2]
  --force            import: re-run even if this checksum is already imported
  --dry-run          import: run in a transaction, then roll back

Environment: DATABASE_URL (import, status)";

struct Args {
    command: String,
    document: String,
    dir: PathBuf,
    base_url: String,
    options: ImportOptions,
}

fn parse_args() -> Result<Args, String> {
    let mut argv = std::env::args().skip(1);
    let command = argv.next().ok_or("missing command")?;
    let mut document = DEFAULT_DOCUMENT.to_string();
    let mut dir = None;
    let mut base_url = DEFAULT_BASE_URL.to_string();
    let mut options = ImportOptions::default();

    while let Some(arg) = argv.next() {
        let mut value = || argv.next().ok_or(format!("{arg} needs a value"));

        match arg.as_str() {
            "--document" => document = value()?,
            "--dir" => dir = Some(PathBuf::from(value()?)),
            "--base-url" => base_url = value()?,
            "--force" => options.force = true,
            "--dry-run" => options.dry_run = true,
            other => return Err(format!("unknown option '{other}'")),
        }
    }
    let dir = dir.unwrap_or_else(|| PathBuf::from(DEFAULT_ROOT).join(&document));

    Ok(Args {
        command,
        document,
        dir,
        base_url,
        options,
    })
}

fn print_reports(reports: &BTreeMap<String, KindReport>) {
    println!(
        "{:<10} {:>6} {:>9} {:>8} {:>10} {:>6}",
        "kind", "total", "inserted", "updated", "unchanged", "stale"
    );
    for (kind, r) in reports {
        println!(
            "{kind:<10} {:>6} {:>9} {:>8} {:>10} {:>6}",
            r.total, r.inserted, r.updated, r.unchanged, r.stale
        );
    }
}

async fn connect() -> Result<sqlx::PgPool, Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be defined")?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await?;

    srd::MIGRATOR.run(&pool).await?;
    Ok(pool)
}

async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    match args.command.as_str() {
        "fetch" => {
            let client = Open5eClient::new(&args.base_url)?;
            let fetched = client.fetch_snapshot(&args.document).await?;

            fetched.validate().map_err(srd::SrdError::Validation)?;
            let manifest = snapshot::write(&fetched, &args.dir)?;
            println!("snapshot {} -> {}", manifest.checksum, args.dir.display());
            for (kind, file) in &manifest.files {
                println!("  {kind:<10} {:>5} entries", file.count);
            }
        }
        "verify" => {
            let snap = snapshot::read(&args.dir)?;
            println!(
                "OK {} ({}) checksum {}",
                snap.manifest.document, snap.manifest.document_name, snap.manifest.checksum
            );
        }
        "rehash" => {
            let manifest = snapshot::rehash(&args.dir)?;
            println!("manifest updated, checksum {}", manifest.checksum);
        }
        "status" => {
            let snap = snapshot::read(&args.dir)?;
            let pool = connect().await?;
            let last = srd::importer::last_checksum(&pool, &snap.manifest.document).await?;
            println!("snapshot : {}", snap.manifest.checksum);
            println!(
                "database : {}",
                last.as_deref().unwrap_or("<never imported>")
            );
            if last.as_deref() != Some(snap.manifest.checksum.as_str()) {
                println!("=> run `srd-import import`");
            }
        }
        "import" => {
            let snap = snapshot::read(&args.dir)?;
            let pool = connect().await?;

            match srd::import(&pool, &snap, args.options).await? {
                ImportOutcome::UpToDate { checksum } => {
                    println!("already up to date ({checksum}), use --force to re-run");
                }
                ImportOutcome::DryRun { reports } => {
                    print_reports(&reports);
                    println!("dry run: rolled back");
                }
                ImportOutcome::Imported { reports } => {
                    print_reports(&reports);
                    println!(
                        "imported {} ({})",
                        snap.manifest.document, snap.manifest.checksum
                    );
                }
            }
        }
        other => return Err(format!("unknown command '{other}'\n\n{USAGE}").into()),
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("error: {e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
