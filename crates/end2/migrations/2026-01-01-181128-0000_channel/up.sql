create table channel (
    id uuid default uuidv7() primary key, -- using UUIDv7 so no need for timestamp col
    sender uuid not null references "user"(id),
    receiver uuid not null references "user"(id)
)