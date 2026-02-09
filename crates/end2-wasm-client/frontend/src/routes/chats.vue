<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { request, type ApiError } from '../api'
import { useUserStore } from '../stores/user'
import ErrorMessage from '../components/ErrorMessage.vue'
import { useChannelStore } from '../stores/channel'

const channelStore = useChannelStore()
const userStore = useUserStore()

const creating_channel = ref(false)
const error = ref<ApiError | null>(null)
const username = ref('')

async function onSubmit() {
    error.value = null
    creating_channel.value = true

    const response = await request('/channel', 'POST', {
        recipient: username.value
    })

    if (!response.ok) {
        error.value = response.error
    }

    creating_channel.value = false
    username.value = ''
}

onMounted(() => {
    channelStore.fetchChannels()
})
</script>

<template>
    <div v-if="userStore.logged_in" class="dashboard">
        <h3>Create chat</h3>
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
                <button type="submit" :disabled="creating_channel">
                    {{ creating_channel ? 'Creating chat' : 'Create chat' }}
                </button>
            </form>
        </div>

        <div class="channel-list">
            <h3>Channels</h3>
            <ul v-if="channelStore.channel_list.length > 0">
                <li v-for="channel_id in channelStore.channel_list" :key="channel_id">
                    <RouterLink :to="`/chat/${channel_id}`">
                        {{ channelStore.getParticipantNames(channel_id) }}
                    </RouterLink>
                </li>
            </ul>
            <div v-else>
                <p>No channels</p>
            </div>
        </div>
    </div>
    <ErrorMessage
        v-if="error"
        :status="error.status"
        :message="error.message"
        :detail="error.detail">
    </ErrorMessage>
</template>
