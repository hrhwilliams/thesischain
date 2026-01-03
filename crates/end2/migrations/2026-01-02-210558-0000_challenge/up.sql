create table challenge (
    id uuid default uuidv7() primary key,
    user_id uuid not null references users(id) on delete cascade
)