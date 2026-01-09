create table "user" (
    id uuid default uuidv7() primary key, -- using UUIDv7 so no need for timestamp col
    username text unique not null,
    nickname text default null,
    password text default null
)