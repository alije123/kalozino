create table "players"
(
    "id"                bigint           not null PRIMARY KEY,
    "balance"           double precision not null,
    "timely_last_at"    timestamptz,
    "timely_last_value" double precision,
    "timely_end_at"     timestamptz,
    "last_steal_at"     timestamptz
);

create table "active_custom_voices"
(
    "id"       bigint not null PRIMARY KEY,
    "owner_id" bigint not null references players (id) on delete cascade
);

create table "voice_config"
(
    "id"        uuid              not null PRIMARY KEY,
    "user_id"   bigint            not null references players (id) on delete cascade,
    "parameter" character varying not null,
    "value"     character varying not null
);

create table "twinks"
(
    "id"       uuid   not null PRIMARY KEY,
    "user_id"  bigint not null references players (id) on delete cascade,
    "twink_id" bigint not null references players (id) on delete cascade
);

create table "shop"
(
    "id"          uuid              not null PRIMARY KEY,
    "name"        character varying not null,
    "price"       double precision  not null,
    "description" character varying not null,
    "item_type"   character varying not null,
    "role_id"     bigint            not null
);

create table "history_journal"
(
    "id"            uuid              not null PRIMARY KEY,
    "user_id"       bigint            not null,
    "at"            timestamptz       not null,
    "value"         double precision  not null,
    "changed_by_id" bigint references players (id),
    "reason"        character varying not null
);

create table "config"
(
    "key"       character varying not null,
    "server_id" bigint            not null,
    "data"      jsonb             not null,
    PRIMARY KEY ("key", "server_id")
);

create table "starboard_messages"
(
    "message_id"           bigint primary key not null,
    "server_id"            bigint             not null,
    "forwarded_message_id" bigint             not null,
    "last_reaction_count"  smallint           not null
);
