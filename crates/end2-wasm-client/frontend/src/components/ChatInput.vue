<script setup lang="ts">
import { computed, onMounted, ref, toRef } from 'vue';
import { db } from '../db';
import { v7 } from 'uuid';
import { useWebSocketStore } from '../stores/socket';
import { useDeviceStore } from '../stores/device';
import { useChannelStore } from '../stores/channel';
import type { EncryptedMessage } from '../types/message';

const props = defineProps({
    channel_id: { type: String, required: true }
})
const channel_id = toRef(props, 'channel_id')
const message_input = ref('');
const devices = computed(() => channel_store.get_devices(channel_id.value))

const socket = useWebSocketStore()
const channel_store = useChannelStore()
const device_store = useDeviceStore()

async function send_message() {
    if (message_input.value.trim().length === 0) {
        return
    }

    let message: EncryptedMessage = {
        message_id: v7(),
        device_id: device_store.device_id()!,
        channel_id: channel_id.value,
        payloads: [],
    }

    for (const device of devices.value) {
        if (device.device_id !== device_store.device_id()) {
            const payload = await device_store.encrypt(channel_id.value, device, message_input.value.trim())
            if (payload.ok) {
                message.payloads.push(payload.value)
            } else {
                console.error(payload.error)
            }
        }
    }

    socket.send(message)
    message_input.value = ''
}

onMounted(() => {
    channel_store.fetch_channel(channel_id.value)
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