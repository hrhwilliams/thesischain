export type DecryptedMessage = {
    message_id: string,
    channel_id: string,
    author_id: string,
    plaintext: string,
    timestamp: Date,
}

export type InboundChatMessage = {
    message_id: string,
    author_id: string,
    device_id: string,
    channel_id: string,
    ciphertext: string,
    timestamp: string,
    is_pre_key: boolean,
}

export type EncryptedMessagePayload = {
    recipient_device_id: string,
    ciphertext: string,
    is_pre_key: boolean,
}

export type EncryptedMessage = {
    message_id: string,
    device_id: string,
    channel_id: string,
    payloads: EncryptedMessagePayload[],
}

export type MessageReceivedEvent = {
    message_id: string,
    channel_id: string,
    timestamp: string,
}
