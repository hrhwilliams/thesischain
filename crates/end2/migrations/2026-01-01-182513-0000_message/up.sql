create table message (
    id uuid default uuidv7() primary key, -- using UUIDv7 so no need for timestamp col
    author_id uuid not null references "user"(id),
    channel_id uuid not null references channel(id) on delete cascade,
    content bytea not null,
    pre_key boolean not null,
    relayed boolean not null default false
)