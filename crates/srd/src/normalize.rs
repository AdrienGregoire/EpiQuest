/*
 ** EPITECH PROJECT, 2026
 ** normalize.rs
 ** File description:
 ** Open5e v2 JSON -> normalized SRD entries
 */

use crate::error::{Result, SrdError};
use crate::model::{Class, Item, Monster, Species, Spell};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

const DROPPED_FIELDS: [&str; 3] = ["key", "document", "crossreferences"];

fn err(kind: &'static str, key: &str, reason: impl Into<String>) -> SrdError {
    SrdError::Normalize {
        kind,
        key: key.to_string(),
        reason: reason.into(),
    }
}

fn key_of(raw: &Value) -> &str {
    raw.get("key").and_then(Value::as_str).unwrap_or("<no key>")
}

pub fn slug_from_key(key: &str, document: &str) -> String {
    key.strip_prefix(&format!("{document}_"))
        .unwrap_or(key)
        .to_string()
}

fn str_field(raw: &Value, field: &str) -> Option<String> {
    raw.get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn named_field(raw: &Value, field: &str) -> Option<String> {
    raw.get(field).and_then(|v| str_field(v, "name"))
}

fn required(kind: &'static str, raw: &Value, value: Option<String>, field: &str) -> Result<String> {
    value.ok_or_else(|| err(kind, key_of(raw), format!("missing field '{field}'")))
}

fn clean_data(raw: &Value) -> Value {
    let mut map: Map<String, Value> = raw.as_object().cloned().unwrap_or_default();

    for field in DROPPED_FIELDS {
        map.remove(field);
    }
    Value::Object(map)
}

fn base(kind: &'static str, raw: &Value, document: &str) -> Result<(String, String)> {
    let key = raw
        .get("key")
        .and_then(Value::as_str)
        .ok_or_else(|| err(kind, "<no key>", "missing field 'key'"))?;
    let name = required(kind, raw, str_field(raw, "name"), "name")?;

    Ok((slug_from_key(key, document), name))
}

fn sort_by_slug<T>(mut entries: Vec<T>, slug: impl Fn(&T) -> &str) -> Vec<T> {
    entries.sort_by(|a, b| slug(a).cmp(slug(b)));
    entries
}

pub fn spells(raws: &[Value], document: &str) -> Result<Vec<Spell>> {
    let mut out = Vec::with_capacity(raws.len());

    for raw in raws {
        let (slug, name) = base("spell", raw, document)?;
        let level = raw
            .get("level")
            .and_then(Value::as_i64)
            .ok_or_else(|| err("spell", key_of(raw), "missing field 'level'"))?;
        let flag = |f: &str| raw.get(f).and_then(Value::as_bool).unwrap_or(false);
        let components: Vec<&str> = [("V", "verbal"), ("S", "somatic"), ("M", "material")]
            .into_iter()
            .filter(|(_, field)| flag(field))
            .map(|(letter, _)| letter)
            .collect();

        out.push(Spell {
            slug,
            name,
            level: level as i16,
            school: required("spell", raw, named_field(raw, "school"), "school")?,
            casting_time: required("spell", raw, str_field(raw, "casting_time"), "casting_time")?,
            range: str_field(raw, "range_text").unwrap_or_else(|| "Self".to_string()),
            duration: required("spell", raw, str_field(raw, "duration"), "duration")?,
            components: components.join(", "),
            data: clean_data(raw),
        });
    }
    Ok(sort_by_slug(out, |s| &s.slug))
}

pub fn monsters(raws: &[Value], document: &str) -> Result<Vec<Monster>> {
    let mut out = Vec::with_capacity(raws.len());

    for raw in raws {
        let (slug, name) = base("monster", raw, document)?;
        let challenge_rating = raw
            .get("challenge_rating")
            .or_else(|| raw.get("challenge_rating_decimal"))
            .and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse().ok()))
            .ok_or_else(|| err("monster", key_of(raw), "missing field 'challenge_rating'"))?;

        out.push(Monster {
            slug,
            name,
            challenge_rating,
            size: required("monster", raw, named_field(raw, "size"), "size")?,
            monster_type: required("monster", raw, named_field(raw, "type"), "type")?,
            data: clean_data(raw),
        });
    }
    Ok(sort_by_slug(out, |m| &m.slug))
}

pub fn items(mundane: &[Value], magic: &[Value], document: &str) -> Result<Vec<Item>> {
    let mut by_slug: BTreeMap<String, Item> = BTreeMap::new();

    for (raws, is_magic) in [(mundane, false), (magic, true)] {
        for raw in raws {
            let (slug, name) = base("item", raw, document)?;
            let mut data = clean_data(raw);

            data["magic"] = Value::Bool(is_magic);
            let item = Item {
                slug: slug.clone(),
                name,
                item_type: named_field(raw, "category").unwrap_or_else(|| "Gear".to_string()),
                rarity: named_field(raw, "rarity"),
                data,
            };
            if by_slug.insert(slug.clone(), item).is_some() {
                tracing::warn!("item '{slug}' exists as mundane and magic item, keeping magic one");
            }
        }
    }
    Ok(by_slug.into_values().collect())
}

fn parse_hit_die(value: &str) -> Option<i16> {
    value
        .to_ascii_lowercase()
        .rsplit('d')
        .next()?
        .trim()
        .parse()
        .ok()
}

pub fn classes(raws: &[Value], document: &str) -> Result<Vec<Class>> {
    let parent_key = |raw: &Value| -> Option<String> {
        raw.get("subclass_of")
            .and_then(|p| p.get("key"))
            .and_then(Value::as_str)
            .map(|k| slug_from_key(k, document))
    };
    let mut classes: BTreeMap<String, Class> = BTreeMap::new();

    for raw in raws.iter().filter(|r| parent_key(r).is_none()) {
        let (slug, name) = base("class", raw, document)?;
        let hit_die = str_field(raw, "hit_dice")
            .as_deref()
            .and_then(parse_hit_die)
            .ok_or_else(|| err("class", key_of(raw), "missing or invalid 'hit_dice'"))?;
        let mut data = clean_data(raw);

        data["subclasses"] = Value::Array(Vec::new());
        classes.insert(
            slug.clone(),
            Class {
                slug,
                name,
                hit_die,
                data,
            },
        );
    }
    let mut subclasses: Vec<&Value> = raws.iter().filter(|r| parent_key(r).is_some()).collect();

    subclasses.sort_by_key(|r| key_of(r).to_string());
    for raw in subclasses {
        let parent = parent_key(raw).unwrap_or_default();
        let class = classes
            .get_mut(&parent)
            .ok_or_else(|| err("class", key_of(raw), format!("unknown parent '{parent}'")))?;
        let mut sub = clean_data(raw);

        sub["slug"] = Value::String(slug_from_key(key_of(raw), document));
        if let Some(list) = class.data["subclasses"].as_array_mut() {
            list.push(sub);
        }
    }
    Ok(classes.into_values().collect())
}

fn trait_desc(raw: &Value, trait_type: &str) -> Option<String> {
    raw.get("traits")?
        .as_array()?
        .iter()
        .find(|t| t.get("type").and_then(Value::as_str) == Some(trait_type))
        .and_then(|t| str_field(t, "desc"))
}

fn first_number(text: &str) -> Option<i32> {
    let digits: String = text
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();

    digits.parse().ok()
}

fn first_size(text: &str) -> Option<String> {
    text.split_whitespace()
        .next()
        .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()).to_string())
        .filter(|w| !w.is_empty())
}

pub fn species(raws: &[Value], document: &str) -> Result<Vec<Species>> {
    let is_sub = |raw: &Value| raw.get("is_subspecies").and_then(Value::as_bool) == Some(true);
    let mut species: BTreeMap<String, Species> = BTreeMap::new();

    for raw in raws.iter().filter(|r| !is_sub(r)) {
        let (slug, name) = base("species", raw, document)?;
        let speed = trait_desc(raw, "SPEED")
            .as_deref()
            .and_then(first_number)
            .unwrap_or_else(|| {
                tracing::warn!("species '{slug}' has no SPEED trait, defaulting to 30");
                30
            });
        let size = trait_desc(raw, "SIZE")
            .as_deref()
            .and_then(first_size)
            .unwrap_or_else(|| "Medium".to_string());
        let mut data = clean_data(raw);

        data["subspecies"] = Value::Array(Vec::new());
        species.insert(
            slug.clone(),
            Species {
                slug,
                name,
                speed,
                size,
                data,
            },
        );
    }
    for raw in raws.iter().filter(|r| is_sub(r)) {
        let parent = raw
            .get("subspecies_of")
            .and_then(|p| p.get("key"))
            .and_then(Value::as_str)
            .map(|k| slug_from_key(k, document))
            .unwrap_or_default();
        let entry = species
            .get_mut(&parent)
            .ok_or_else(|| err("species", key_of(raw), format!("unknown parent '{parent}'")))?;
        let mut sub = clean_data(raw);

        sub["slug"] = Value::String(slug_from_key(key_of(raw), document));
        if let Some(list) = entry.data["subspecies"].as_array_mut() {
            list.push(sub);
        }
    }
    Ok(species.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const DOC: &str = "srd-2024";

    #[test]
    fn slug_strips_document_prefix() {
        assert_eq!(slug_from_key("srd-2024_acid-arrow", DOC), "acid-arrow");
        assert_eq!(slug_from_key("homebrew", DOC), "homebrew");
    }

    #[test]
    fn spell_is_normalized() {
        let raw = json!({
            "key": "srd-2024_acid-arrow", "name": "Acid Arrow", "level": 2,
            "school": {"name": "Evocation", "key": "evocation"},
            "casting_time": "action", "range_text": "90 feet", "duration": "instantaneous",
            "verbal": true, "somatic": true, "material": true,
            "document": {"key": "srd-2024"}, "crossreferences": {"to": []},
        });
        let spell = &spells(&[raw], DOC).unwrap()[0];

        assert_eq!(spell.slug, "acid-arrow");
        assert_eq!(spell.school, "Evocation");
        assert_eq!(spell.components, "V, S, M");
        assert!(spell.data.get("document").is_none());
        assert!(spell.data.get("key").is_none());
    }

    #[test]
    fn spell_missing_school_fails() {
        let raw = json!({"key": "srd-2024_x", "name": "X", "level": 1,
            "casting_time": "action", "duration": "1 minute"});

        assert!(spells(&[raw], DOC).is_err());
    }

    #[test]
    fn monster_fractional_cr() {
        let raw = json!({"key": "srd-2024_rat", "name": "Rat", "challenge_rating": 0.125,
            "size": {"name": "Tiny"}, "type": {"name": "Beast"}});

        assert_eq!(monsters(&[raw], DOC).unwrap()[0].challenge_rating, 0.125);
    }

    #[test]
    fn magic_item_wins_on_collision() {
        let mundane = json!({"key": "srd-2024_potion", "name": "Potion",
            "category": {"name": "Potion"}});
        let magic = json!({"key": "srd-2024_potion", "name": "Potion",
            "category": {"name": "Potion"}, "rarity": {"name": "Common"}});
        let items = items(&[mundane], &[magic], DOC).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].rarity.as_deref(), Some("Common"));
        assert_eq!(items[0].data["magic"], json!(true));
    }

    #[test]
    fn subclasses_are_nested() {
        let fighter = json!({"key": "srd-2024_fighter", "name": "Fighter",
            "hit_dice": "D10", "subclass_of": null});
        let champion = json!({"key": "srd-2024_champion", "name": "Champion",
            "subclass_of": {"key": "srd-2024_fighter"}});
        let classes = classes(&[champion, fighter], DOC).unwrap();

        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].hit_die, 10);
        assert_eq!(classes[0].data["subclasses"][0]["slug"], json!("champion"));
    }

    #[test]
    fn species_speed_and_size_from_traits() {
        let raw = json!({"key": "srd-2024_human", "name": "Human", "is_subspecies": false,
        "traits": [
            {"type": "SIZE", "desc": "Medium (about 4-7 feet tall) or Small"},
            {"type": "SPEED", "desc": "30 feet"}
        ]});
        let species = &species(&[raw], DOC).unwrap()[0];

        assert_eq!(species.speed, 30);
        assert_eq!(species.size, "Medium");
    }
}
