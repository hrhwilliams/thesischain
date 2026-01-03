create table otks (
    id uuid default uuidv7() primary key,
    user_id uuid not null references users(id) on delete cascade,
    otk bytea not null check(length(otk) = 32)
)