<script setup lang="ts">
import { onMounted, toRef } from 'vue'
import { from, useObservable } from '@vueuse/rxjs'
import { liveQuery, Dexie } from 'dexie'
import { db } from '../db'
import { useUserStore } from '../stores/user'
import { useChannelStore } from '../stores/channel'

const props = defineProps({
    channel_id: { type: String, required: true }
})

const channel_id = toRef(props, 'channel_id')
const user_store = useUserStore()
const channel_store = useChannelStore()

const messages = useObservable(from(
    liveQuery(async () => {
        if (!channel_id.value) return []

        return await db.messages
            .where('[channel_id+timestamp]')
            .between(
                [channel_id.value, Dexie.minKey], 
                [channel_id.value, Dexie.maxKey]
            )
            .toArray()
    }))
)

onMounted(async () => {
    await channel_store.fetch_chat_history(channel_id.value)
})
</script>

<template>
    <div class="message-history">
        <div v-for="message in messages" :key="message.message_id" class="message">
            <span class="author">{{ user_store.get_display_name(message.author_id) }}</span> <span class="date">{{ new Date(message.timestamp).toLocaleTimeString() }}</span>
            <div class="content">{{ message.plaintext }}</div>
        </div>
    </div>
</template>

<style>
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
</style>