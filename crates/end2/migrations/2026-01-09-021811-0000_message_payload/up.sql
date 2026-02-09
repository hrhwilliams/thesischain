create table message_payload (
    message_id uuid not null references message(id) on delete cascade,
    recipient_device_id uuid not null references device(id) on delete cascade,
    ciphertext bytea not null,
    is_pre_key boolean not null,
    primary key (message_id, recipient_device_id)
)