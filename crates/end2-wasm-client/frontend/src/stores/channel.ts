import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { request } from "../api";
import { useUserStore } from "./user";
import type { ChannelInfo } from "../types/channel";

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

    return {
        channel_list,
        add_channel,
        fetch_channels,
        get_participants,
    }
})