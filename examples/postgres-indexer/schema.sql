-- One row per decoded transfer. The primary key makes a repeated update a no-op.
CREATE TABLE IF NOT EXISTS token_transfers (
    signature   TEXT    NOT NULL,
    ix_path     TEXT    NOT NULL,  -- position in the transaction, for example "0" or "2.1"
    slot        BIGINT  NOT NULL,
    source      TEXT    NOT NULL,
    destination TEXT    NOT NULL,
    amount      NUMERIC NOT NULL,  -- u64 does not fit in BIGINT
    PRIMARY KEY (signature, ix_path)
);

-- Highest slot written so far. Used to choose where to resume after a restart.
CREATE TABLE IF NOT EXISTS indexer_checkpoint (
    id   SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    slot BIGINT   NOT NULL
);
