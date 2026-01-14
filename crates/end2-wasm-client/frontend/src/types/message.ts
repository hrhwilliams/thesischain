export type DecryptedMessage = {
    message_id: string,
    channel_id: string,
    author_id: string,
    plaintext: string,
    timestamp: Date
}

export type InboundChatMessage = {
    message_id: string,
    device_id: string,
    channel_id: string,
    ciphertext: string,
    timestamp: string,
    is_pre_key: boolean,
}

