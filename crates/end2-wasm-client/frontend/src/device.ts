import { DeviceContext } from '../../pkg/end2_wasm_client'

export type InboundChatMessage = {
    message_id: string,
    device_id: string,
    channel_id: string,
    ciphertext: string,
    timestamp: string,
    is_pre_key: boolean,
}

export type ChatMessage = {
    message_id: string,
    author_id: string,
    plaintext: string,
    timestamp: string,
}

export type DeviceContextKeys = {
    x25519: string,
    ed25519: string,
    signature: string,
}

export type DeviceOneTimeKeys = {
    keys: [string],
    signature: string,
}
