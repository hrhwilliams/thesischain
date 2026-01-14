<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { request, type ApiError } from '../api'
import { useUserStore } from '../stores/user'
import ErrorMessage from '../components/ErrorMessage.vue'
import { useChannelStore } from '../stores/channel'

const channel_store = useChannelStore()
const user_store = useUserStore()

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

// const { } = useQuery({
//     queryKey: ['channels'],
//     enabled: computed(() => user_store.logged_in),
//     queryFn: async () => {
//         await channel_store.fetch_channels()
//         return channel_store.channel_list
//     }
// })

onMounted(() => {
    channel_store.fetch_channels()
})
</script>

<template>
    <div v-if="user_store.logged_in" class="dashboard">
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
            <ul v-if="channel_store.channel_list.length > 0">
                <li v-for="channel_id in channel_store.channel_list" :key="channel_id">
                    <RouterLink :to="`/chat/${channel_id}`">
                        {{ channel_store.get_participants(channel_id) }}
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
