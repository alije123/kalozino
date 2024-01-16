CREATE TABLE "players" (
    "id" bigint NOT NULL PRIMARY KEY,
    "balance" double precision NOT NULL,
    "timely_last_at" timestamptz,
    "timely_last_value" double precision,
    "timely_end_at" timestamptz,
    "last_steal_at" timestamptz
);

CREATE TABLE "active_custom_voices" (
    "id" bigint NOT NULL PRIMARY KEY,
    "owner_id" bigint NOT NULL REFERENCES players (id) ON DELETE CASCADE
);

CREATE TABLE "voice_config" (
    "id" uuid NOT NULL PRIMARY KEY,
    "user_id" bigint NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    "parameter" character varying NOT NULL,
    "value" character varying NOT NULL
);

CREATE TABLE "twinks" (
    "id" uuid NOT NULL PRIMARY KEY,
    "user_id" bigint NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    "twink_id" bigint NOT NULL REFERENCES players (id) ON DELETE CASCADE
);

CREATE TABLE "shop" (
    "id" uuid NOT NULL PRIMARY KEY,
    "name" character varying NOT NULL,
    "price" double precision NOT NULL,
    "description" character varying NOT NULL,
    "item_type" character varying NOT NULL,
    "role_id" bigint NOT NULL
);

CREATE TABLE "history_journal" (
    "id" uuid NOT NULL PRIMARY KEY,
    "user_id" bigint NOT NULL,
    "at" timestamptz NOT NULL,
    "value" double precision NOT NULL,
    "changed_by_id" bigint REFERENCES players (id),
    "reason" character varying NOT NULL
);

CREATE TABLE "config" (
    "key" character varying NOT NULL PRIMARY KEY,
    "data" jsonb NOT NULL
)
