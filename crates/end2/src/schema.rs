// @generated automatically by Diesel CLI.

diesel::table! {
    challenge (id) {
        id -> Uuid,
        user_id -> Uuid,
    }
}

diesel::table! {
    channel (id) {
        id -> Uuid,
        sender -> Uuid,
        receiver -> Uuid,
    }
}

diesel::table! {
    message (id) {
        id -> Uuid,
        author_id -> Uuid,
        channel_id -> Uuid,
        content -> Bytea,
        pre_key -> Bool,
        relayed -> Bool,
    }
}

diesel::table! {
    one_time_key (id) {
        id -> Uuid,
        user_id -> Uuid,
        otk -> Bytea,
    }
}

diesel::table! {
    session (id) {
        id -> Uuid,
        user_id -> Uuid,
    }
}

diesel::table! {
    user (id) {
        id -> Uuid,
        username -> Text,
        ed25519 -> Bytea,
        curve25519 -> Bytea,
        signature -> Bytea,
    }
}

diesel::joinable!(challenge -> user (user_id));
diesel::joinable!(message -> channel (channel_id));
diesel::joinable!(message -> user (author_id));
diesel::joinable!(one_time_key -> user (user_id));
diesel::joinable!(session -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    challenge,
    channel,
    message,
    one_time_key,
    session,
    user,
);
