// @generated automatically by Diesel CLI.

diesel::table! {
    challenge (id) {
        id -> Uuid,
        user_id -> Uuid,
    }
}

diesel::table! {
    messages (id) {
        id -> Uuid,
        author -> Uuid,
        room_id -> Uuid,
        content -> Text,
    }
}

diesel::table! {
    otks (id) {
        id -> Uuid,
        user_id -> Uuid,
        otk -> Bytea,
    }
}

diesel::table! {
    room_participants (room_id, user_id) {
        room_id -> Uuid,
        user_id -> Uuid,
    }
}

diesel::table! {
    rooms (id) {
        id -> Uuid,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        ed25519 -> Bytea,
        curve25519 -> Bytea,
        signature -> Bytea,
    }
}

diesel::joinable!(challenge -> users (user_id));
diesel::joinable!(messages -> rooms (room_id));
diesel::joinable!(messages -> users (author));
diesel::joinable!(otks -> users (user_id));
diesel::joinable!(room_participants -> rooms (room_id));
diesel::joinable!(room_participants -> users (user_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    challenge,
    messages,
    otks,
    room_participants,
    rooms,
    sessions,
    users,
);
