import type { DeviceInfo } from "./device"
import type { UserInfo } from "./user"

export type ChannelInfo = {
    channel_id: string,
    participants: UserInfo[],
    devices: DeviceInfo[],
}

export type ChannelId = {
    channel_id: string,
}

export type ChatMessage = {
    message_id: string,
    author_id: string,
    plaintext: string,
    timestamp: string,
}
