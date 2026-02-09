create table message (
    id uuid primary key,
    sender_id uuid not null references "user"(id) on delete cascade,
    sender_device_id uuid not null references device(id) on delete cascade,
    channel_id uuid not null references channel(id) on delete cascade,
    created timestamptz not null default now()
)