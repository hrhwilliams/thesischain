create table discord_info (
    id uuid default uuidv7() primary key,
    user_id uuid unique not null references "user"(id) on delete cascade,
    discord_id bigint unique not null,
    discord_username text unique not null,
    global_name text,
    avatar text
)