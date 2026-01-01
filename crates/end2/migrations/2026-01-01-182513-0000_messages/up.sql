create table messages (
    id uuid default uuidv7() primary key, -- using UUIDv7 so no need for timestamp col
    author uuid not null references users(id),
    room_id uuid not null references rooms(id) on delete cascade,
    content text not null
)