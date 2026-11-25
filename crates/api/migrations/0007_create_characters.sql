CREATE TABLE characters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id),
    name VARCHAR(100) NOT NULL,
    class_id UUID NOT NULL REFERENCES classes(id),
    species_id UUID NOT NULL REFERENCES species(id),
    level SMALLINT NOT NULL DEFAULT 1,
    hp_current INTEGER NOT NULL,
    hp_max INTEGER NOT NULL,
    armor_class INTEGER NOT NULL DEFAULT 10,
    data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX characters_campaign_id_idx ON characters(campaign_id);
CREATE INDEX characters_owner_id_idx ON characters(owner_id);

CREATE TABLE character_spells (
    character_id UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    spell_id UUID NOT NULL REFERENCES spells(id) ON DELETE CASCADE,
    prepared BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (character_id, spell_id)
);

CREATE TABLE character_items (
    character_id UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (character_id, item_id)
);
