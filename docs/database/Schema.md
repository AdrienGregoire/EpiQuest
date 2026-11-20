# Field dictionary EpiQuest

PostgreSQL schema documentation.
Migrations: `crates/api/migrations/0001` → `0009`.

## Conventions
| Rule | Detail |
|------|--------|
| PK | `id UUID PRIMARY KEY DEFAULT gen_random_uuid()` |
| Timestamps | `TIMESTAMPTZ NOT NULL DEFAULT NOW()` |
| Flexible SRD data | `data JSONB NOT NULL DEFAULT '{}'` |
| Naming | English (consistent with `users`, `campaigns`) |
| Cascade | deleting a campaign removes related entities |

## Relations
    users
     └── campaigns (gm_id)
           ├── campaign_members / campaign_invites
           ├── characters
           │     ├── character_spells → spells
           │     └── character_items  → items
           ├── maps
           │     ├── map_layers
           │     └── pawns → characters? / monsters?
           ├── encounters → combats
           └── rolls → combats? / characters?
    Global catalog:
      classes, species, spells, items, monsters

---

## Existing tables

### `users` `0001`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| username | VARCHAR(50) | UNIQUE NOT NULL | Display name |
| email | TEXT | UNIQUE NOT NULL | Email |
| password_hash | TEXT | NOT NULL | Password hash |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |

### `sessions` `0002`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| user_id | UUID | FK → users CASCADE | Owner |
| token_hash | TEXT | UNIQUE NOT NULL | Session token hash |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |
| expires_at | TIMESTAMPTZ | NOT NULL | Expiration |
| revoked_at | TIMESTAMPTZ | NULL | Revocation time |

### `campaigns` `0003`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| gm_id | UUID | FK → users CASCADE | Game master |
| name | VARCHAR(100) | NOT NULL | Campaign name |
| description | TEXT | NOT NULL DEFAULT '' | Description |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Updated at |

### `campaign_members` / `campaign_invites` `0004`
Members use a composite PK (`campaign_id` + `user_id`). Invites use a unique `code`.

---

## SRD catalog
### `classes` `0005`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| slug | VARCHAR(100) | UNIQUE NOT NULL | Stable SRD key (`fighter`) |
| name | VARCHAR(100) | NOT NULL | Display name |
| hit_die | SMALLINT | NOT NULL | Hit die (6/8/10/12) |
| data | JSONB | NOT NULL DEFAULT '{}' | Features, saves… |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |

### `species` `0005`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| slug | VARCHAR(100) | UNIQUE NOT NULL | Stable SRD key (`elf`) |
| name | VARCHAR(100) | NOT NULL | Display name |
| speed | INTEGER | NOT NULL | Movement speed |
| size | VARCHAR(20) | NOT NULL | Medium, Small… |
| data | JSONB | NOT NULL DEFAULT '{}' | Traits, ASI… |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |

### `spells` `0006`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| slug | VARCHAR(150) | UNIQUE NOT NULL | Stable SRD key |
| name | VARCHAR(150) | NOT NULL | Spell name |
| level | SMALLINT | NOT NULL | 0 = cantrip, 1–9 |
| school | VARCHAR(50) | NOT NULL | Magic school |
| casting_time | TEXT | NOT NULL | Casting time |
| range | TEXT | NOT NULL | Range |
| duration | TEXT | NOT NULL | Duration |
| components | TEXT | NOT NULL DEFAULT '' | V/S/M |
| data | JSONB | NOT NULL DEFAULT '{}' | Description, upcast… |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |
Index: `spells_level_idx (level)`

### `items` `0006`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| slug | VARCHAR(150) | UNIQUE NOT NULL | Stable SRD key |
| name | VARCHAR(150) | NOT NULL | Item name |
| item_type | VARCHAR(50) | NOT NULL | weapon, armor, gear, magic… |
| rarity | VARCHAR(50) | NULL | common, rare… |
| data | JSONB | NOT NULL DEFAULT '{}' | Stats, effects… |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |
Index: `items_item_type_idx (item_type)`

### `monsters` `0006`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| slug | VARCHAR(150) | UNIQUE NOT NULL | Stable SRD key |
| name | VARCHAR(150) | NOT NULL | Monster name |
| challenge_rating | NUMERIC(4,2) | NOT NULL | CR |
| size | VARCHAR(20) | NOT NULL | Size |
| monster_type | VARCHAR(50) | NOT NULL | beast, undead… |
| data | JSONB | NOT NULL DEFAULT '{}' | Full stat block |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |
Index: `monsters_cr_idx (challenge_rating)`
---
## Character

### `characters` `0007`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| campaign_id | UUID | FK → campaigns CASCADE | Campaign |
| owner_id | UUID | FK → users | Owner player |
| name | VARCHAR(100) | NOT NULL | Character name |
| class_id | UUID | FK → classes | Class |
| species_id | UUID | FK → species | Species |
| level | SMALLINT | NOT NULL DEFAULT 1 | Level |
| hp_current | INTEGER | NOT NULL | Current HP |
| hp_max | INTEGER | NOT NULL | Max HP |
| armor_class | INTEGER | NOT NULL DEFAULT 10 | AC |
| data | JSONB | NOT NULL DEFAULT '{}' | Abilities, skills… |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Updated at |
Indexes: `characters_campaign_id_idx`, `characters_owner_id_idx`

### `character_spells` `0007`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| character_id | UUID | FK → characters CASCADE, PK | Character |
| spell_id | UUID | FK → spells CASCADE, PK | Spell |
| prepared | BOOLEAN | NOT NULL DEFAULT false | Prepared |

### `character_items` `0007`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| character_id | UUID | FK → characters CASCADE, PK | Character |
| item_id | UUID | FK → items CASCADE, PK | Item |
| quantity | INTEGER | NOT NULL DEFAULT 1 | Quantity |
---

## Map / VTT

### `maps` `0008`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| campaign_id | UUID | FK → campaigns CASCADE | Campaign |
| name | VARCHAR(100) | NOT NULL | Map name |
| width | INTEGER | NOT NULL | Width (cells) |
| height | INTEGER | NOT NULL | Height (cells) |
| grid_size | INTEGER | NOT NULL DEFAULT 70 | Pixels per cell |
| background_url | TEXT | NULL | Background image |
| data | JSONB | NOT NULL DEFAULT '{}' | Walls, fog… |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Updated at |
Index: `maps_campaign_id_idx`

### `map_layers` `0008`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| map_id | UUID | FK → maps CASCADE | Parent map |
| name | VARCHAR(100) | NOT NULL | Layer name |
| layer_type | VARCHAR(50) | NOT NULL | background, tokens, fog, walls, dm… |
| z_index | INTEGER | NOT NULL DEFAULT 0 | Draw order |
| visible | BOOLEAN | NOT NULL DEFAULT true | Visibility |
| data | JSONB | NOT NULL DEFAULT '{}' | Layer content |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |
Index: `map_layers_map_id_idx`

### `pawns` `0008`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| map_id | UUID | FK → maps CASCADE | Map |
| layer_id | UUID | FK → map_layers SET NULL, NULL | Layer |
| character_id | UUID | FK → characters SET NULL, NULL | Linked character |
| monster_id | UUID | FK → monsters SET NULL, NULL | Linked monster |
| name | VARCHAR(100) | NOT NULL | Token label |
| x | INTEGER | NOT NULL DEFAULT 0 | Grid X |
| y | INTEGER | NOT NULL DEFAULT 0 | Grid Y |
| rotation | REAL | NOT NULL DEFAULT 0 | Degrees |
| size | SMALLINT | NOT NULL DEFAULT 1 | Cells occupied |
| visible_to_players | BOOLEAN | NOT NULL DEFAULT true | Player visibility |
| data | JSONB | NOT NULL DEFAULT '{}' | Metadata |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Updated at |
Index: `pawns_map_id_idx`
---

## Encounter / combat / roll

### `encounters` `0009`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| campaign_id | UUID | FK → campaigns CASCADE | Campaign |
| map_id | UUID | FK → maps SET NULL, NULL | Linked map |
| name | VARCHAR(100) | NOT NULL | Name |
| status | VARCHAR(20) | NOT NULL DEFAULT 'draft' | draft / ready / active / completed |
| data | JSONB | NOT NULL DEFAULT '{}' | Composition, GM notes |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Created at |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Updated at |
Index: `encounters_campaign_id_idx`

### `combats` `0009`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| encounter_id | UUID | FK → encounters CASCADE | Encounter |
| campaign_id | UUID | FK → campaigns CASCADE | Campaign |
| map_id | UUID | FK → maps SET NULL, NULL | Map |
| status | VARCHAR(20) | NOT NULL DEFAULT 'active' | active / paused / ended |
| round | INTEGER | NOT NULL DEFAULT 1 | Current round |
| current_turn_index | INTEGER | NOT NULL DEFAULT 0 | Initiative index |
| started_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Start time |
| ended_at | TIMESTAMPTZ | NULL | End time |
| data | JSONB | NOT NULL DEFAULT '{}' | Initiative order, state |
Indexes: `combats_campaign_id_idx`, `combats_encounter_id_idx`

### `rolls` `0009`
| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | UUID | PK | Identifier |
| campaign_id | UUID | FK → campaigns CASCADE | Campaign |
| combat_id | UUID | FK → combats SET NULL, NULL | Linked combat |
| user_id | UUID | FK → users | Who rolled |
| character_id | UUID | FK → characters SET NULL, NULL | Linked character |
| formula | TEXT | NOT NULL | e.g. `1d20+5` |
| total | INTEGER | NOT NULL | Final result |
| roll_type | VARCHAR(50) | NOT NULL | attack, save, check, damage, custom |
| is_private | BOOLEAN | NOT NULL DEFAULT false | GM-only roll |
| data | JSONB | NOT NULL DEFAULT '{}' | Dice breakdown |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Roll time |
Indexes: `rolls_campaign_id_idx`, `rolls_combat_id_idx`
---

## Ticket entity → table mapping
| Entity | Table |
|--------|-------|
| campaign | `campaigns` |
| character | `characters` |
| class | `classes` |
| species | `species` |
| spell | `spells` |
| item | `items` |
| monster | `monsters` |
| map | `maps` |
| layer | `map_layers` |
| pawn | `pawns` |
| encounter | `encounters` |
| combat | `combats` |
| roll | `rolls` |
