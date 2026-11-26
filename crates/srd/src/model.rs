/*
 ** EPITECH PROJECT, 2026
 ** model.rs
 ** File description:
 ** normalized SRD entries (our own format, independent of Open5e)
 */

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spell {
    pub slug: String,
    pub name: String,
    pub level: i16,
    pub school: String,
    pub casting_time: String,
    pub range: String,
    pub duration: String,
    pub components: String,
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub slug: String,
    pub name: String,
    pub item_type: String,
    pub rarity: Option<String>,
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Monster {
    pub slug: String,
    pub name: String,
    pub challenge_rating: f64,
    pub size: String,
    pub monster_type: String,
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Class {
    pub slug: String,
    pub name: String,
    pub hit_die: i16,
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Species {
    pub slug: String,
    pub name: String,
    pub speed: i32,
    pub size: String,
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileEntry {
    pub count: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub format_version: u32,
    pub document: String,
    pub document_name: String,
    pub source: String,
    pub source_url: Option<String>,
    pub license: String,
    pub generated_at: String,
    pub files: BTreeMap<String, FileEntry>,
    pub checksum: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub manifest: Manifest,
    pub classes: Vec<Class>,
    pub species: Vec<Species>,
    pub spells: Vec<Spell>,
    pub items: Vec<Item>,
    pub monsters: Vec<Monster>,
}

fn check_slugs<'a>(
    kind: &str,
    entries: impl Iterator<Item = (&'a str, &'a str)>,
    max_len: usize,
    errors: &mut Vec<String>,
) {
    let mut seen = HashSet::new();

    for (slug, name) in entries {
        if slug.is_empty() || slug.len() > max_len {
            errors.push(format!("{kind}: invalid slug '{slug}'"));
        }
        if name.trim().is_empty() || name.len() > max_len {
            errors.push(format!("{kind} '{slug}': invalid name '{name}'"));
        }
        if !seen.insert(slug) {
            errors.push(format!("{kind}: duplicate slug '{slug}'"));
        }
    }
}

impl Snapshot {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        check_slugs(
            "class",
            self.classes
                .iter()
                .map(|c| (c.slug.as_str(), c.name.as_str())),
            100,
            &mut errors,
        );
        check_slugs(
            "species",
            self.species
                .iter()
                .map(|s| (s.slug.as_str(), s.name.as_str())),
            100,
            &mut errors,
        );
        check_slugs(
            "spell",
            self.spells
                .iter()
                .map(|s| (s.slug.as_str(), s.name.as_str())),
            150,
            &mut errors,
        );
        check_slugs(
            "item",
            self.items
                .iter()
                .map(|i| (i.slug.as_str(), i.name.as_str())),
            150,
            &mut errors,
        );
        check_slugs(
            "monster",
            self.monsters
                .iter()
                .map(|m| (m.slug.as_str(), m.name.as_str())),
            150,
            &mut errors,
        );
        for c in &self.classes {
            if ![6, 8, 10, 12].contains(&c.hit_die) {
                errors.push(format!("class '{}': invalid hit die {}", c.slug, c.hit_die));
            }
        }
        for s in &self.species {
            if s.speed <= 0 || s.size.is_empty() || s.size.len() > 20 {
                errors.push(format!("species '{}': invalid speed/size", s.slug));
            }
        }
        for s in &self.spells {
            if !(0..=9).contains(&s.level) || s.school.is_empty() || s.school.len() > 50 {
                errors.push(format!("spell '{}': invalid level/school", s.slug));
            }
        }
        for i in &self.items {
            if i.item_type.is_empty() || i.item_type.len() > 50 {
                errors.push(format!("item '{}': invalid item_type", i.slug));
            }
        }
        for m in &self.monsters {
            if !(0.0..=30.0).contains(&m.challenge_rating) {
                errors.push(format!("monster '{}': invalid CR", m.slug));
            }
            if m.size.is_empty() || m.size.len() > 20 || m.monster_type.len() > 50 {
                errors.push(format!("monster '{}': invalid size/type", m.slug));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
