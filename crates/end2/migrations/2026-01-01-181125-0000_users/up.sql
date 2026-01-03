create table users (
    id uuid default uuidv7() primary key, -- using UUIDv7 so no need for timestamp col
    username text unique not null,
    ed25519 bytea not null check(length(ed25519) = 32),
    curve25519 bytea not null check(length(curve25519)  = 32),
    signature bytea not null check(length(curve25519)  = 64)
)