create table "user" (
    id uuid default uuidv7() primary key, -- using UUIDv7 so no need for timestamp col
    username text unique not null check(length(username) < 20),
    nickname text default null check(length(username) < 20),
    password text default null
)