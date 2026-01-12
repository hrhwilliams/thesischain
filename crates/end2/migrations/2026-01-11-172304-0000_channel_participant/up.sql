create table channel_participant (
    channel_id uuid not null references channel(id) on delete cascade,
    user_id uuid not null references "user"(id) on delete cascade,
    primary key (channel_id, user_id)
)