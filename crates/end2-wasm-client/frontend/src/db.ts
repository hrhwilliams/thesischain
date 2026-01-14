import Dexie, { type EntityTable } from "dexie"
import type { DecryptedMessage } from "./types/message"

export type DevicePickle = {
    user_id: string,
    pickle: string
}

export type SessionPickle = {
    channel_device_id: string,
    pickle: string,
}

export const db = new Dexie('messages') as Dexie & {
    messages: EntityTable<DecryptedMessage, 'message_id'>
    sessions: EntityTable<SessionPickle, 'channel_device_id'>
    account: EntityTable<DevicePickle, 'user_id'>
}

db.version(1).stores({
    messages: 'message_id, [channel_id+timestamp], author_id',
    sessions: 'channel_device_id',
    account: 'user_id'
})
