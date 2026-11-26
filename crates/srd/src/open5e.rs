/*
** EPITECH PROJECT, 2026
** EpiQuest
** File description:
** open5e.rs
*/

use crate::error::{Result, SrdError};
use crate::model::{FORMAT_VERSION, Manifest, Snapshot};
use crate::normalize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Duration;

pub const DEFAULT_BASE_URL: &str = "https://api.open5e.com/v2";
const PAGE_SIZE: u32 = 100;
const MAX_PAGES: usize = 500;
const MAX_RETRIES: u32 = 3;

pub struct Open5eClient {
    http: reqwest::Client,
    base_url: String,
}

impl Open5eClient {
    pub fn new(base_url: &str) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent("epiquest-srd-importer")
            .timeout(Duration::from_secs(60))
            .build()?;

        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    async fn get_page(&self, url: &str) -> Result<Value> {
        let mut attempt = 0;

        loop {
            attempt += 1;
            let result = async {
                self.http
                    .get(url)
                    .send()
                    .await?
                    .error_for_status()?
                    .json::<Value>()
                    .await
            }
            .await;
            match result {
                Ok(page) => return Ok(page),
                Err(e) if attempt < MAX_RETRIES => {
                    tracing::warn!("GET {url} failed ({e}), retry {attempt}/{MAX_RETRIES}");
                    tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    pub async fn fetch_all(&self, endpoint: &str, document: &str) -> Result<Vec<Value>> {
        let mut url = Some(format!(
            "{}/{endpoint}/?document__key__in={document}&limit={PAGE_SIZE}",
            self.base_url
        ));
        let mut results = Vec::new();
        let mut pages = 0;

        while let Some(current) = url {
            pages += 1;
            if pages > MAX_PAGES {
                return Err(SrdError::Open5e(format!("{endpoint}: too many pages")));
            }
            let page = self.get_page(&current).await?;
            let entries = page
                .get("results")
                .and_then(Value::as_array)
                .ok_or_else(|| SrdError::Open5e(format!("{endpoint}: no 'results' array")))?;

            results.extend(
                entries
                    .iter()
                    .filter(|e| e["document"]["key"].as_str() == Some(document))
                    .cloned(),
            );
            url = page.get("next").and_then(Value::as_str).map(str::to_string);
        }
        tracing::info!("{endpoint}: {} entries", results.len());
        Ok(results)
    }

    async fn document_name(&self, document: &str) -> Result<String> {
        let url = format!("{}/documents/{document}/", self.base_url);
        let doc = self.get_page(&url).await?;

        Ok(doc
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(document)
            .to_string())
    }

    pub async fn fetch_snapshot(&self, document: &str) -> Result<Snapshot> {
        let classes = self.fetch_all("classes", document).await?;
        let species = self.fetch_all("species", document).await?;
        let spells = self.fetch_all("spells", document).await?;
        let items = self.fetch_all("items", document).await?;
        let magic_items = self.fetch_all("magicitems", document).await?;
        let monsters = self.fetch_all("creatures", document).await?;

        Ok(Snapshot {
            manifest: Manifest {
                format_version: FORMAT_VERSION,
                document: document.to_string(),
                document_name: self.document_name(document).await?,
                source: "open5e".to_string(),
                source_url: Some(self.base_url.clone()),
                license: "CC-BY-4.0".to_string(),
                generated_at: chrono::Utc::now().to_rfc3339(),
                files: BTreeMap::new(),
                checksum: String::new(),
            },
            classes: normalize::classes(&classes, document)?,
            species: normalize::species(&species, document)?,
            spells: normalize::spells(&spells, document)?,
            items: normalize::items(&items, &magic_items, document)?,
            monsters: normalize::monsters(&monsters, document)?,
        })
    }
}
