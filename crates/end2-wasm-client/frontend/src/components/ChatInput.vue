<script setup lang="ts">
import { onMounted, ref, toRef } from 'vue';
import { useChannelStore } from '../stores/channel';
import { useMessageStore } from '../stores/message';

const props = defineProps({
    channel_id: { type: String, required: true }
})
const channel_id = toRef(props, 'channel_id')
const message_input = ref('');

const channelStore = useChannelStore()
const messageStore = useMessageStore()

async function send_message() {
    const text = message_input.value.trim()
    if (text.length === 0) {
        return
    }

    const result = await messageStore.sendMessage(channel_id.value, text)
    if (!result.ok) {
        console.error('failed to send message:', result.error)
        return
    }

    message_input.value = ''
}

onMounted(() => {
    channelStore.fetchChannel(channel_id.value)
})
</script>

<template>
    <div class="message-box">
        <form @submit.prevent="send_message" class="input-area">
            <input type="text" v-model="message_input" id="message-input" name="message-input" autocomplete="off" required>
            <button type="submit">Send</button>
        </form>
    </div>
</template>

<style>
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
