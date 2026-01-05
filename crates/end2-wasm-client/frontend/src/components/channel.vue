<script setup>
import {computed, onMounted, ref, watch, nextTick, onUnmounted} from 'vue';
import { useRoute } from 'vue-router';
import { useAuth } from '../common.js';
import { authService } from '../auth.js'

const { user } = useAuth();

const route = useRoute();

const channelId = route.params.id;
const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const ws = new WebSocket(`${protocol}//localhost:8081/api/channel/ws/${channelId}`);
const messageInput = ref("");
const messages = ref([]);

ws.onmessage = async (event) => {
    console.log("got event" + event.data);
    const json = JSON.parse(event.data);

    if (json.pre_key) {
        try {
            const plaintext = await authService.decrypt_new_session(channelId, json);
            console.log(plaintext);
        } catch (e) {
            console.log(e);
        }
    } else {
        try {
            const plaintext = authService.decrypt(channelId, json);
            console.log(plaintext);
        } catch (e) {
            console.log(e);
        }
    }
};

const handleSendMessage = async () => {
    if (messageInput.value.length === 0) return;

    try {
        const outboundMsg = authService.encrypt(channelId, messageInput.value);
        console.log(JSON.stringify(outboundMsg));
        ws.send(JSON.stringify(outboundMsg));
    } catch (e) {
        console.log(e);
    }

    messageInput.value = "";
};

onMounted(async () => {
    if (!authService.channel_has_session(channelId)) {
        await authService.create_session_in_channel(channelId);
    } else {
        console.log("found existing session");
    }
})

onUnmounted(() => {
    ws.close();
});
</script>

<template>
    <h2>hmm</h2>
    <div class="chat-container">
        <div class="message-history">
            <div class="message"><span class="author">Maki</span> <span class="date">1-4-2026 4:39 PM</span><br>Here is my first message. Hello, world! I sure do find encryption to be very neat. Lorem ipsum dolor sit amet consectetur adipisicing elit. Dignissimos illo neque nam illum dolore laboriosam amet quisquam officiis modi quibusdam et deleniti, maiores libero ad, delectus, quos voluptatibus blanditiis ratione.</div>
            <div>message 2</div>
            <div>message 3</div>
        </div>

        <form @submit.prevent="handleSendMessage" class="input-area">
            <input type="text" v-model="messageInput" id="message-input" name="message-input" autocomplete="off" required>
            <button type="submit">Send</button>
        </form>
    </div>
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