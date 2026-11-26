/*
** EPITECH PROJECT, 2026
** EpiQuest
** File description:
** snapshot.rs
*/

use crate::error::{Result, SrdError};
use crate::model::{FORMAT_VERSION, FileEntry, Manifest, Snapshot};
use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const MANIFEST_FILE: &str = "manifest.json";
pub const KINDS: [&str; 5] = ["classes", "species", "spells", "items", "monsters"];

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    to_hex(&Sha256::digest(bytes))
}

pub fn global_checksum(files: &BTreeMap<String, FileEntry>) -> String {
    let joined: String = files
        .iter()
        .map(|(kind, entry)| format!("{kind}:{}\n", entry.sha256))
        .collect();

    sha256_hex(joined.as_bytes())
}

fn io_err(path: &Path, source: std::io::Error) -> SrdError {
    SrdError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn json_err(path: &Path, source: serde_json::Error) -> SrdError {
    SrdError::Json {
        path: path.to_path_buf(),
        source,
    }
}

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("SRD entries are always serializable");

    bytes.push(b'\n');
    bytes
}

fn encoded_files(snapshot: &Snapshot) -> Vec<(&'static str, usize, Vec<u8>)> {
    vec![
        ("classes", snapshot.classes.len(), encode(&snapshot.classes)),
        ("species", snapshot.species.len(), encode(&snapshot.species)),
        ("spells", snapshot.spells.len(), encode(&snapshot.spells)),
        ("items", snapshot.items.len(), encode(&snapshot.items)),
        (
            "monsters",
            snapshot.monsters.len(),
            encode(&snapshot.monsters),
        ),
    ]
}

pub fn write(snapshot: &Snapshot, dir: &Path) -> Result<Manifest> {
    fs::create_dir_all(dir).map_err(|e| io_err(dir, e))?;
    let files = encoded_files(snapshot);
    let entries: BTreeMap<String, FileEntry> = files
        .iter()
        .map(|(kind, count, bytes)| {
            let entry = FileEntry {
                count: *count,
                sha256: sha256_hex(bytes),
            };
            (kind.to_string(), entry)
        })
        .collect();
    let checksum = global_checksum(&entries);

    if let Ok(existing) = read_manifest(dir)
        && existing.checksum == checksum
    {
        tracing::info!("snapshot unchanged ({checksum}), nothing written");
        return Ok(existing);
    }
    for (kind, _, bytes) in &files {
        let path = dir.join(format!("{kind}.json"));

        fs::write(&path, bytes).map_err(|e| io_err(&path, e))?;
    }
    let manifest = Manifest {
        files: entries,
        checksum,
        ..snapshot.manifest.clone()
    };
    let path = dir.join(MANIFEST_FILE);

    fs::write(&path, encode(&manifest)).map_err(|e| io_err(&path, e))?;
    Ok(manifest)
}

pub fn read_manifest(dir: &Path) -> Result<Manifest> {
    let path = dir.join(MANIFEST_FILE);
    let bytes = fs::read(&path).map_err(|e| io_err(&path, e))?;

    serde_json::from_slice(&bytes).map_err(|e| json_err(&path, e))
}

fn read_kind<T: DeserializeOwned>(dir: &Path, kind: &str, manifest: &Manifest) -> Result<Vec<T>> {
    let path = dir.join(format!("{kind}.json"));
    let bytes = fs::read(&path).map_err(|e| io_err(&path, e))?;
    let actual = sha256_hex(&bytes);
    let expected = manifest
        .files
        .get(kind)
        .map(|f| f.sha256.clone())
        .unwrap_or_default();

    if actual != expected {
        return Err(SrdError::ChecksumMismatch {
            file: format!("{kind}.json"),
            expected,
            actual,
        });
    }
    serde_json::from_slice(&bytes).map_err(|e| json_err(&path, e))
}

pub fn read(dir: &Path) -> Result<Snapshot> {
    let manifest = read_manifest(dir)?;

    if manifest.format_version != FORMAT_VERSION {
        return Err(SrdError::FormatVersion {
            found: manifest.format_version,
            expected: FORMAT_VERSION,
        });
    }
    let checksum = global_checksum(&manifest.files);

    if checksum != manifest.checksum {
        return Err(SrdError::ChecksumMismatch {
            file: MANIFEST_FILE.to_string(),
            expected: manifest.checksum.clone(),
            actual: checksum,
        });
    }
    let snapshot = Snapshot {
        classes: read_kind(dir, "classes", &manifest)?,
        species: read_kind(dir, "species", &manifest)?,
        spells: read_kind(dir, "spells", &manifest)?,
        items: read_kind(dir, "items", &manifest)?,
        monsters: read_kind(dir, "monsters", &manifest)?,
        manifest,
    };
    snapshot.validate().map_err(SrdError::Validation)?;
    Ok(snapshot)
}

pub fn rehash(dir: &Path) -> Result<Manifest> {
    let mut manifest = read_manifest(dir)?;

    for kind in KINDS {
        let path = dir.join(format!("{kind}.json"));
        let bytes = fs::read(&path).map_err(|e| io_err(&path, e))?;
        let parsed: Vec<serde_json::Value> =
            serde_json::from_slice(&bytes).map_err(|e| json_err(&path, e))?;

        manifest.files.insert(
            kind.to_string(),
            FileEntry {
                count: parsed.len(),
                sha256: sha256_hex(&bytes),
            },
        );
    }
    manifest.checksum = global_checksum(&manifest.files);
    manifest.generated_at = chrono::Utc::now().to_rfc3339();
    let path = dir.join(MANIFEST_FILE);

    fs::write(&path, encode(&manifest)).map_err(|e| io_err(&path, e))?;
    read(dir)?;
    Ok(manifest)
}
