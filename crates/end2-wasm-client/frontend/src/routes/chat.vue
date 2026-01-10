<script setup lang="ts">
import { computed, ref } from 'vue';
import { useQuery } from '@tanstack/vue-query'
import { useClientState } from '../state'
import { useRoute } from 'vue-router'
import { get_channel_info } from '../api'
import type { ApiError } from '../api';

const error = ref<ApiError | null>(null)
const state = useClientState()
const route = useRoute()

const channel_id = route.params.chat_id as string;
const message_input = ref<string>("")

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
        state.register_channel(response.data)

        return response.data
    },
})
</script>

<template>
    <div class="chat-container">
        <div class="message-history">
            <!-- <div class="message"><span class="author">Maki</span> <span class="date">1-4-2026 4:39 PM</span><br>Here is my first message. Hello, world! I sure do find encryption to be very neat. Lorem ipsum dolor sit amet consectetur adipisicing elit. Dignissimos illo neque nam illum dolore laboriosam amet quisquam officiis modi quibusdam et deleniti, maiores libero ad, delectus, quos voluptatibus blanditiis ratione.</div> -->
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

.message-history > .message {
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