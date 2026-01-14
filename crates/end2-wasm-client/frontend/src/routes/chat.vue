<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRoute } from 'vue-router'
import type { ApiError } from '../api';
import ChatHistory from '../components/ChatHistory.vue';
import ChatInput from '../components/ChatInput.vue';
import ErrorMessage from '../components/ErrorMessage.vue';

const error = ref<ApiError | null>(null)
const route = useRoute()
const channel_id = route.params.chat_id as string;
</script>

<template>
    <div class="chat-container">
        <ChatHistory :channel_id="channel_id"></ChatHistory>
        <ChatInput :channel_id="channel_id"></ChatInput>
    </div>
    <ErrorMessage
        v-if="error"
        :status="error.status"
        :message="error.message"
        :detail="error.detail">
    </ErrorMessage>
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
</style>