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