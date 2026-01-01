create table room_participants (
    room_id uuid not null references rooms(id) on delete cascade,
    user_id uuid not null references users(id) on delete cascade,
    primary key (room_id, user_id)
)