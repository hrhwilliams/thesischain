// @generated automatically by Diesel CLI.

diesel::table! {
    channel (id) {
        id -> Uuid,
    }
}

diesel::table! {
    channel_participant (channel_id, user_id) {
        channel_id -> Uuid,
        user_id -> Uuid,
    }
}

diesel::table! {
    device (id) {
        id -> Uuid,
        user_id -> Uuid,
        ed25519 -> Nullable<Bytea>,
        x25519 -> Nullable<Bytea>,
    }
}

diesel::table! {
    discord_auth_token (id) {
        id -> Uuid,
        user_id -> Uuid,
        access_token -> Bytea,
        refresh_token -> Nullable<Bytea>,
        expires -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    discord_info (id) {
        id -> Uuid,
        user_id -> Uuid,
        discord_id -> Int8,
        discord_username -> Text,
        global_name -> Nullable<Text>,
        avatar -> Nullable<Text>,
    }
}

diesel::table! {
    message (id) {
        id -> Uuid,
        sender_id -> Uuid,
        sender_device_id -> Uuid,
        channel_id -> Uuid,
        created -> Timestamptz,
    }
}

diesel::table! {
    message_payload (message_id, recipient_device_id) {
        message_id -> Uuid,
        recipient_device_id -> Uuid,
        ciphertext -> Bytea,
        is_pre_key -> Bool,
    }
}

diesel::table! {
    one_time_key (id) {
        id -> Uuid,
        device_id -> Uuid,
        otk -> Bytea,
    }
}

diesel::table! {
    user (id) {
        id -> Uuid,
        username -> Text,
        nickname -> Nullable<Text>,
        password -> Nullable<Text>,
    }
}

diesel::table! {
    web_session (id) {
        id -> Uuid,
        blob -> Jsonb,
    }
}

diesel::joinable!(channel_participant -> channel (channel_id));
diesel::joinable!(channel_participant -> user (user_id));
diesel::joinable!(device -> user (user_id));
diesel::joinable!(discord_auth_token -> user (user_id));
diesel::joinable!(discord_info -> user (user_id));
diesel::joinable!(message -> channel (channel_id));
diesel::joinable!(message -> device (sender_device_id));
diesel::joinable!(message -> user (sender_id));
diesel::joinable!(message_payload -> device (recipient_device_id));
diesel::joinable!(message_payload -> message (message_id));
diesel::joinable!(one_time_key -> device (device_id));

diesel::allow_tables_to_appear_in_same_query!(
    channel,
    channel_participant,
    device,
    discord_auth_token,
    discord_info,
    message,
    message_payload,
    one_time_key,
    user,
    web_session,
);
