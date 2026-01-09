create table discord_auth_token (
    id uuid default uuidv7() primary key,
    user_id uuid unique not null references "user"(id) on delete cascade,
    access_token bytea not null,
    refresh_token bytea,
    expires timestamptz
)