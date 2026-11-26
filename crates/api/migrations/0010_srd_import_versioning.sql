CREATE TABLE srd_imports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document VARCHAR(50) NOT NULL,
    format_version INTEGER NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    source TEXT NOT NULL,
    counts JSONB NOT NULL DEFAULT '{}',
    imported_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX srd_imports_document_idx ON srd_imports(document, imported_at DESC);

ALTER TABLE classes
    ADD COLUMN source_document VARCHAR(50),
    ADD COLUMN content_hash VARCHAR(64),
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

ALTER TABLE species
    ADD COLUMN source_document VARCHAR(50),
    ADD COLUMN content_hash VARCHAR(64),
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

ALTER TABLE spells
    ADD COLUMN source_document VARCHAR(50),
    ADD COLUMN content_hash VARCHAR(64),
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

ALTER TABLE items
    ADD COLUMN source_document VARCHAR(50),
    ADD COLUMN content_hash VARCHAR(64),
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

ALTER TABLE monsters
    ADD COLUMN source_document VARCHAR(50),
    ADD COLUMN content_hash VARCHAR(64),
    ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

ALTER TABLE monsters ALTER COLUMN challenge_rating TYPE NUMERIC(5, 3);
