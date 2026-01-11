<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useQuery } from '@tanstack/vue-query'
import { useClientState } from '../state'
import { useRoute } from 'vue-router'
import { get_channel_info } from '../api'
import type { ApiError, UserInfo } from '../api';

const error = ref<ApiError | null>(null)
const state = useClientState()
const route = useRoute()

const channel_id = route.params.chat_id as string;
const message_input = ref<string>("")
const messages = computed(() =>
    state.messages[channel_id] ?? []
)

const handleSendMessage = async () => {
    if (message_input.value.length === 0) return

    try {
        const need_otks = state.devices_missing_sessions(channel_id)

        if (need_otks.length > 0) {
            await state.create_sessions(channel_id, need_otks)
        }

        await state.encrypt_and_send(channel_id, message_input.value)
    } catch (e) {
        console.error(e)
        error.value = {
            status: 0,
            message: 'Unexpected',
            detail: e instanceof Error ? e.message : undefined
        }
    } finally {
        message_input.value = ""
    }
};

const { data: channel_info } = useQuery({
    queryKey: ['channels'],
    enabled: computed(() => state.is_logged_in && !state.has_channel_info_for(channel_id)),
    queryFn: async () => {
        const response = await get_channel_info(channel_id)
        if (!response.ok) {
            throw response.error
        }

        console.log("pulled data:", response.data)
        // cache usernames and nicknames here
        state.register_channel(response.data)

        return response.data
    },
})

onMounted(() => {
    state.get_messages(channel_id)
})
</script>

<template>
    <div class="chat-container">
        <div class="message-history">
            <div v-for="message in messages" :key="message.message_id" class="message">
                <span class="author">{{ state.get_user(channel_id, message.author_id) }}</span> <span class="date">{{ new Date(message.timestamp).toLocaleTimeString() }}</span>
                <div class="content">{{ message.plaintext }}</div>
            </div>
        </div>

        <form @submit.prevent="handleSendMessage" class="input-area">
            <input type="text" v-model="message_input" id="message-input" name="message-input" autocomplete="off" required>
            <button type="submit">Send</button>
        </form>
    </div>
    <p v-if="error" class="error">
        <strong>Error response from server</strong><br>{{ error.message }}
        <span v-if="error.detail">: {{ error.detail }}</span>
    </p>
</template>

<style>
.chat-container {
    display: flex;
    flex-direction: column;
    width: 450px;
    height: 720px;
    border: 1px solid #ccc;
    border-radius: 4px;
    overflow: hidden;
}

.message-history {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    background: #fcfcfc;
}

.message-history > .message > .content {
    padding-left: 1em;
    text-indent: -1em;
}

.message-history > .message > .author {
    font-weight: bold;
    font-size: larger;
}
.message-history > .message > .date {
    color: #222;
    font-size: smaller;
}

.input-area {
    display: flex;
    border-top: 1px solid #ccc;
}

.input-area input {
    border: none;
    flex: 1;
    padding: 10px;
    outline: none;
}

.input-area button {
    border: none;
    padding: 0 15px;
    cursor: pointer;
}
</style>