import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { request } from "../api";
import { useUserStore } from "./user";
import type { ChannelInfo } from "../types/channel";
import type { InboundChatMessage, DecryptedMessage } from "../types/message";
import { db } from "../db";
import { useDeviceStore } from "./device";
import Dexie from "dexie";

export type Channel = {
    channel_id: string
}

export const useChannelStore = defineStore('channel', () => {
    const channels = ref<Record<string, ChannelInfo>>({});

    const channel_list = computed(() => Object.keys(channels.value))

    function add_channel(channel_id: string, channel_info: ChannelInfo) {
        channels.value[channel_id] = channel_info
    }

    async function fetch_channel(channel_id: string) {
        if (channels.value[channel_id]) {
            return
        }

        const response = await request<ChannelInfo>(`/channel/${channel_id}`, 'GET')
        if (response.ok) {
            channels.value[channel_id] = response.value
        } else {
            console.error('failed to fetch channel info:', response.error)
        }
    }

    async function fetch_channels() {
        const response = await request<Channel[]>('/me/channels', 'GET')
        if (response.ok) {
            for (const channel of response.value) {
                fetch_channel(channel.channel_id)
            }
        } else {
            console.error('failed to fetch channel info:', response.error)
        }
    }

    function get_participants(channel_id: string): string {
        const user_store = useUserStore()
        const channel = channels.value[channel_id]
        
        if (channel) {
            return channel.participants.map((user) => user_store.get_display_name(user.id))
                .join(', ')
        } else {
            return channel_id
        }
    }

    function get_devices(channel_id: string) {
        const info = channels.value[channel_id]
        if (info) {
            return info.devices
        }

        return []
    }

    async function save_message(decrypted: DecryptedMessage) {
        try {
            const normalized = {
                ...decrypted,
                timestamp: new Date(decrypted.timestamp)
            }

            await db.messages.put(normalized);
        } catch (e) {
            console.error('failed to save message:', e)
            throw e
        }
    }

    async function fetch_chat_history(channel_id: string) {
        const channel_store = useChannelStore()
        const device_store = useDeviceStore()

        const last_message = await db.messages
            .where('[channel_id+timestamp]')
            .between(
                [channel_id, Dexie.minKey], 
                [channel_id, Dexie.maxKey]
            )
            .last();

        if (!device_store.device_id()) {
            return
        }

        let params = new URLSearchParams({ 
            device: device_store.device_id()!
        })
        
        if (last_message) {
            params.append('after', last_message.message_id)
        }

        const response = await request<InboundChatMessage[]>(`/channel/${channel_id}/history?${params}`, 'GET')
        if (response.ok) {
            for (const message of response.value) {
                const decrypted = await device_store.decrypt(message)

                if (decrypted.ok) {
                    await channel_store.save_message(decrypted.value)
                }
            }
        }
    }

    return {
        channel_list,
        add_channel,
        fetch_channels,
        fetch_channel,
        get_participants,
        get_devices,
        save_message,
        fetch_chat_history
    }
})