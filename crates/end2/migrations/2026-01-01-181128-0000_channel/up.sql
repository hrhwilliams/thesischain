create table channel (
    id uuid default uuidv7() primary key, -- using UUIDv7 so no need for timestamp col
    sender_id uuid not null references "user"(id) on delete cascade,
    recipient_id uuid not null references "user"(id) on delete cascade
)