CREATE TABLE encounters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    map_id UUID REFERENCES maps(id) ON DELETE SET NULL,
    name VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX encounters_campaign_id_idx ON encounters(campaign_id);

CREATE TABLE combats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    encounter_id UUID NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    campaign_id UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    map_id UUID REFERENCES maps(id) ON DELETE SET NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    round INTEGER NOT NULL DEFAULT 1,
    current_turn_index INTEGER NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    data JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX combats_campaign_id_idx ON combats(campaign_id);
CREATE INDEX combats_encounter_id_idx ON combats(encounter_id);

CREATE TABLE rolls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    combat_id UUID REFERENCES combats(id) ON DELETE SET NULL,
    user_id UUID NOT NULL REFERENCES users(id),
    character_id UUID REFERENCES characters(id) ON DELETE SET NULL,
    formula TEXT NOT NULL,
    total INTEGER NOT NULL,
    roll_type VARCHAR(50) NOT NULL,
    is_private BOOLEAN NOT NULL DEFAULT false,
    data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX rolls_campaign_id_idx ON rolls(campaign_id);
CREATE INDEX rolls_combat_id_idx ON rolls(combat_id);
