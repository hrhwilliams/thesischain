create table message (
    id uuid default uuidv7() primary key, -- using UUIDv7 so no need for timestamp col
    author uuid not null references "user"(id),
    channel_id uuid not null references channel(id) on delete cascade,
    content bytea not null
)