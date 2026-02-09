import { defineStore } from 'pinia'
import { shallowRef, ref } from 'vue'
import type { DeviceId, DeviceInfo } from '../types/device'
import type { ChannelInfo } from '../types/channel'
import type { EncryptedMessage, InboundChatMessage } from '../types/message'
import { useChannelStore } from './channel'
import { useMessageStore } from './message'

type WsEvent =
    | { counter: number, type: 'channel_created', data: ChannelInfo }
    | { counter: number, type: 'device_added', data: DeviceInfo }
    | { counter: number, type: 'message', data: InboundChatMessage }
    | { counter: number, type: 'ping', data: null }

export const useWebSocketStore = defineStore('socket', () => {
    const ws = shallowRef<WebSocket | null>(null)
    const counter = ref<number>(-1)

    let retry_count = 0
    let reconnect_timer: number | null = null
    let should_reconnect = true

    function connect(deviceId: DeviceId) {
        if (ws.value) {
            const state = ws.value.readyState

            if (state === WebSocket.OPEN || state === WebSocket.CONNECTING) {
                return
            } else {
                ws.value.close()
            }
        }

        ws.value = new WebSocket(`wss://chat.fiatlux.dev/api/me/device/${deviceId}/ws`)

        ws.value.onopen = () => {
            retry_count = 0
        }

        ws.value.onclose = (event) => {
            console.warn('socket closed:', event.reason)

            if (should_reconnect) {
                reconnect(deviceId)
            }
        }

        ws.value.onmessage = async (event) => {
            const channelStore = useChannelStore()
            const messageStore = useMessageStore()
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
                    channelStore.addChannel(payload.data.channel_id, payload.data)
                    break
                case 'message':
                    await messageStore.handleInbound(payload.data)
                    break
                case 'device_added':
                case 'ping':
                    break
            }

            counter.value = payload.counter
        }
    }

    function reconnect(deviceId: DeviceId) {
        if (reconnect_timer !== null) {
            return
        }

        const delay = Math.min(1000 * (2 ** retry_count), 10000)

        reconnect_timer = setTimeout(() => {
            reconnect_timer = null
            retry_count += 1
            connect(deviceId)
        }, delay)
    }

    function send(message: EncryptedMessage) {
        if (!ws.value) {
            console.error('socket null')
            return
        }

        if (ws.value.readyState === WebSocket.OPEN) {
            ws.value.send(JSON.stringify(message))
        } else {
            console.error('socket not open')
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
        disconnect,
    }
})
