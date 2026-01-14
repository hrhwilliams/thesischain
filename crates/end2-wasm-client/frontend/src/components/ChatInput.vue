<script setup lang="ts">
import { ref, toRef } from 'vue';
import { db } from '../db';
import { v7 } from 'uuid';
import { useWebSocketStore } from '../stores/socket';

const props = defineProps({
    channel_id: { type: String, required: true }
})
const channel_id = toRef(props, 'channel_id')
const message_input = ref('');

const socket = useWebSocketStore()

async function send_message() {
    message_input.value = ''
    const uuid = v7()
}
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