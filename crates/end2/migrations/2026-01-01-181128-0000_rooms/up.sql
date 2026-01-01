create table rooms (
    id uuid default uuidv7() primary key -- using UUIDv7 so no need for timestamp col
)