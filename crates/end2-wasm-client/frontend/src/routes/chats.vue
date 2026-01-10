<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useClientState } from '../state'
import { create_channel_with, get_channels } from '../api'
import type { ApiError } from '../api'

const state = useClientState()

const creating = ref(false)
const request_error = ref<ApiError | null>(null)
const username = ref('')

const { data: channels } = useQuery({
    queryKey: ['channels'],
    enabled: computed(() => state.is_logged_in),
    queryFn: async () => {
        const response = await get_channels()
        if (!response.ok) {
            throw response.error
        }
        return response.data
    },
})

watch(channels, (channel_list) => {
    if (!channel_list) return

    for (const channel of channel_list) {
        state.add_channel(channel)
    }
})

async function onSubmit() {
    request_error.value = null

    try {
        creating.value = true

        const response = await create_channel_with(username.value)
        if (!response.ok) {
            request_error.value = response.error
        }
    } catch (e) {

    } finally {
        creating.value = false
    }
}
</script>

<template>
    <h3>Create chat</h3>
    <div v-if="state.is_logged_in" class = "dashboard">
    <div class="message-user-form">
        <form @submit.prevent="onSubmit">
        <label>
            Username
            <input
            v-model="username"
            type="text"
            autocomplete="username"
            required
            />
        </label>
        <br>
        <button type="submit" :disabled="creating">
            {{ creating ? 'Creating chat' : 'Create chat' }}
        </button>
    </form>
    <p v-if="request_error" class="error">
        <strong>Error response from server</strong><br>{{ request_error.message }}
        <span v-if="request_error.detail">: {{ request_error.detail }}</span>
    </p>
    </div>
    <div class="channel-list">
        <h3>Channels</h3>
        <ul v-if="state.channel_list.length > 0">
            <li v-for="channel in state.channel_list">
                <RouterLink :to="`/chat/${ channel.channel_id }`">{{ channel.username }}</RouterLink>
            </li>
        </ul>
        <div v-else>
            <p>No channels</p>
        </div>
    </div>
    </div>
</template>