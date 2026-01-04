create table challenge (
    id uuid default uuidv7() primary key,
    user_id uuid not null references "user"(id) on delete cascade
)