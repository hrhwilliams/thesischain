// @generated automatically by Diesel CLI.

diesel::table! {
    message_requests (id) {
        id -> Uuid,
        sender -> Uuid,
        receiver -> Uuid,
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
        pass -> Text,
    }
}

diesel::joinable!(messages -> rooms (room_id));
diesel::joinable!(messages -> users (author));
diesel::joinable!(room_participants -> rooms (room_id));
diesel::joinable!(room_participants -> users (user_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    message_requests,
    messages,
    room_participants,
    rooms,
    sessions,
    users,
);
