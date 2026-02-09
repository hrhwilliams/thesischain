import { defineStore } from "pinia"
import { computed, ref } from "vue"
import { request } from "../api"
import { useUserStore } from "./user"
import type { ChannelInfo } from "../types/channel"
import type { DeviceInfo } from "../types/device"

export type Channel = {
    channel_id: string
}

export const useChannelStore = defineStore('channel', () => {
    const channels = ref<Record<string, ChannelInfo>>({})

    const channel_list = computed(() => Object.keys(channels.value))

    function addChannel(channelId: string, channelInfo: ChannelInfo) {
        channels.value[channelId] = channelInfo
    }

    async function fetchChannel(channelId: string) {
        if (channels.value[channelId]) {
            return
        }

        const response = await request<ChannelInfo>(`/channel/${channelId}`, 'GET')
        if (response.ok) {
            channels.value[channelId] = response.value
        } else {
            console.error('failed to fetch channel info:', response.error)
        }
    }

    async function fetchChannels() {
        const response = await request<Channel[]>('/me/channels', 'GET')
        if (response.ok) {
            for (const channel of response.value) {
                fetchChannel(channel.channel_id)
            }
        } else {
            console.error('failed to fetch channels:', response.error)
        }
    }

    function getParticipantNames(channelId: string): string {
        const userStore = useUserStore()
        const channel = channels.value[channelId]

        if (channel) {
            return channel.participants
                .map((user) => userStore.get_display_name(user.id))
                .join(', ')
        }

        return channelId
    }

    function getDevices(channelId: string): DeviceInfo[] {
        const info = channels.value[channelId]
        if (info) {
            return info.devices
        }
        return []
    }

    return {
        channels,
        channel_list,
        addChannel,
        fetchChannel,
        fetchChannels,
        getParticipantNames,
        getDevices,
    }
})
