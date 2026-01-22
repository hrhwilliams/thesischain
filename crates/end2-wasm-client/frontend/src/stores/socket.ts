import { defineStore } from 'pinia'
import { shallowRef, ref } from 'vue'
import type { DeviceId, DeviceInfo } from '../types/device'
import type { ChannelInfo } from '../types/channel'
import type { EncryptedMessage, InboundChatMessage } from '../types/message'
import { useChannelStore } from './channel'
import { useDeviceStore } from './device'

type WsEvent =
    | { counter: number, type: 'channel_created', data: ChannelInfo }
    | { counter: number, type: 'device_added', data: DeviceInfo }
    | { counter: number, type: 'message', data: InboundChatMessage }
    | { counter: number, type: 'ping', data: null }
    // | { type: 'message_received', data: MessageReceivedReply }
    // | { type: 'nickname_changed', data: NewNickname }

export const useWebSocketStore = defineStore('socket', () => {
    const ws = shallowRef<WebSocket | null>(null)
    const counter = ref<number>(-1)

    let retry_count = 0
    let reconnect_timer: number | null = null
    let should_reconnect = true

    function connect(device_id: DeviceId) {
        if (ws.value) {
            const state = ws.value.readyState

            if (state === WebSocket.OPEN || state === WebSocket.CONNECTING) {
                return
            } else {
                ws.value.close()
            }
        }

        ws.value = new WebSocket(`wss://chat.fiatlux.dev/api/me/device/${device_id}/ws`)

        ws.value.onopen = () => {
            retry_count = 0
        }

        ws.value.onclose = (event) => {
            console.warn('socket closed: ', event.reason)

            if (should_reconnect) {
                reconnect(device_id)
            }
        }

        ws.value.onmessage = async (event) => {
            const channel_store = useChannelStore()
            const device_store = useDeviceStore()
            const payload = JSON.parse(event.data) as WsEvent

            if (payload.counter !== counter.value + 1) {
                if (ws.value) {
                    ws.value.send(`{"replay":${counter.value}}`)
                } else {
                    console.error('failed to send replay message because ws closed')
                }
                return
            }

            switch (payload.type) {
                case 'channel_created':
                    channel_store.add_channel(payload.data.channel_id, payload.data)
                    break
                case 'device_added':
                    break
                case 'message':
                    console.info('message', payload.data)
                    const decrypted = await device_store.decrypt(payload.data)
                    if (decrypted.ok) {
                        await channel_store.save_message(decrypted.value)
                        console.log(decrypted.value.plaintext)
                    }
                    break
                case 'ping':
                    break
                // case 'message_received':
                //     break
                // case 'nickname_changed':
                //     break
            }

            counter.value = payload.counter
        }
    }

    function reconnect(device_id: DeviceId) {
        if (reconnect_timer !== null) {
            return
        }

        const delay = Math.min(1000 * (2 ** retry_count), 10000)

        reconnect_timer = setTimeout(() => {
            reconnect_timer = null
            retry_count += 1
            connect(device_id)
        },
        delay)
    }

    function send(message: EncryptedMessage) {
        if (ws.value) {
            const state = ws.value.readyState

            if (state === WebSocket.OPEN) {
                const msg = JSON.stringify(message)
                console.log('sending ', msg)
                ws.value!.send(msg)
            } else {
                console.error('socket not open')
            }
        } else {
            console.error('socket null')
        }
    }

    function disconnect() {
        ws.value?.close()
        ws.value = null
        should_reconnect = false
    }

    return {
        connect,
        send,
        disconnect
    }
})
