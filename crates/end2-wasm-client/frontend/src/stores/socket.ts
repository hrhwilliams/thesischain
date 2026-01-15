import { defineStore } from 'pinia';
import { markRaw } from 'vue';
import type { DeviceId, DeviceInfo } from '../types/device';
import type { ChannelInfo } from '../types/channel';
import type { EncryptedMessage, InboundChatMessage } from '../types/message';
import { useUserStore } from './user';
import { useChannelStore } from './channel';
import { useDeviceStore } from './device';

type WsEvent =
    | { type: 'channel_created', data: ChannelInfo }
    | { type: 'device_added', data: DeviceInfo }
    | { type: 'message', data: InboundChatMessage }
    // | { type: 'message_received', data: MessageReceivedReply }
    // | { type: 'nickname_changed', data: NewNickname }

export const useWebSocketStore = defineStore('socket', {
    state: () => ({
        ws: null as WebSocket | null,
    }),

    actions: {
        connect(device_id: DeviceId) {
            this.ws = markRaw(new WebSocket(`ws://localhost:8081/api/me/device/${device_id}/ws`))

            this.ws.onmessage = async (ev) => {
                const user_store = useUserStore()
                const channel_store = useChannelStore()
                const device_store = useDeviceStore()
                const payload = JSON.parse(ev.data) as WsEvent

                switch (payload.type) {
                    case 'channel_created':
                        channel_store.add_channel(payload.data.channel_id, payload.data)
                        break;
                    case 'device_added':
                        break;
                    case 'message':
                        console.info('message', payload.data)
                        const decrypted = await device_store.decrypt(payload.data)
                        if (decrypted.ok) {
                            console.log(decrypted.value.plaintext)
                        }
                        break;
                    // case 'message_received':
                    //     break;
                    // case 'nickname_changed':
                    //     break;
                }
            }
        },

        send(message: EncryptedMessage) {
            if (this.ws) {
                const msg = JSON.stringify(message)
                this.ws.send(msg)
            }
        },

        disconnect() {
            if (this.ws) {
                this.ws.close()
            }
        }
    }
})
