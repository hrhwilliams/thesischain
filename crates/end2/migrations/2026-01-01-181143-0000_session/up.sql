create table session (
    id uuid default uuidv7() primary key, -- using UUIDv7 so no need for timestamp col
    user_id uuid not null references "user"(id) on delete cascade
)