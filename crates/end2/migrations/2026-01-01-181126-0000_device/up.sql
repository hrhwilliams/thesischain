create table device (
    id uuid default uuidv7() primary key,
    user_id uuid not null references "user"(id) on delete cascade,
    ed25519 bytea check(length(ed25519) = 32),
    x25519  bytea check(length(x25519)  = 32)
)