create table one_time_key (
    id uuid default uuidv7() primary key,
    device_id uuid not null references device(id) on delete cascade,
    otk bytea not null check(length(otk) = 32)
)