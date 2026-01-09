create table message (
    id uuid default uuidv7() primary key, -- using UUIDv7 so no need for timestamp col
    sender_id uuid not null references "user"(id),
    sender_device uuid not null references device(id),
    channel_id uuid not null references channel(id) on delete cascade
)