CREATE TABLE spells (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(150) NOT NULL UNIQUE,
    name VARCHAR(150) NOT NULL,
    level SMALLINT NOT NULL,
    school VARCHAR(50) NOT NULL,
    casting_time TEXT NOT NULL,
    range TEXT NOT NULL,
    duration TEXT NOT NULL,
    components TEXT NOT NULL DEFAULT '',
    data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX spells_level_idx ON spells(level);

CREATE TABLE items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(150) NOT NULL UNIQUE,
    name VARCHAR(150) NOT NULL,
    item_type VARCHAR(50) NOT NULL,
    rarity VARCHAR(50),
    data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX items_item_type_idx ON items(item_type);

CREATE TABLE monsters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(150) NOT NULL UNIQUE,
    name VARCHAR(150) NOT NULL,
    challenge_rating NUMERIC(4, 2) NOT NULL,
    size VARCHAR(20) NOT NULL,
    monster_type VARCHAR(50) NOT NULL,
    data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX monsters_cr_idx ON monsters(challenge_rating);
