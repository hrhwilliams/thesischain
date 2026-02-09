<script setup lang="ts">
import { nextTick, toRef, ref, watch, computed } from 'vue'
import { from, useObservable } from '@vueuse/rxjs'
import { liveQuery, Dexie } from 'dexie'
import { db } from '../db'
import { useUserStore } from '../stores/user'
import { useMessageStore } from '../stores/message'
import { useQuery } from '@tanstack/vue-query'

const props = defineProps({
    channel_id: { type: String, required: true }
})

const channel_id = toRef(props, 'channel_id')
const userStore = useUserStore()
const messageStore = useMessageStore()
const history = ref<HTMLDivElement | null>(null)

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

watch(messages, async (new_msgs, old_msgs) => {
    const el = history.value
    if (!el || !new_msgs) {
        return
    }

    const is_at_bottom = el.scrollHeight - el.scrollTop - el.clientHeight <= 50
    const is_first_load = !old_msgs || old_msgs.length === 0

    await nextTick()

    if (is_first_load || is_at_bottom) {
        el.scrollTop = el.scrollHeight
    }
})

useQuery({
    queryKey: ['chat-history', channel_id.value],
    enabled: computed(() => userStore.logged_in),
    queryFn: async () => {
        await messageStore.fetchHistory(channel_id.value)
    },
})
</script>

<template>
    <div ref="history" class="message-history">
        <div v-for="message in messages" :key="message.message_id" class="message">
            <div v-if="userStore.has_nickname(message.author_id)">
                <span class="author">{{ userStore.get_display_name(message.author_id) }} ({{ userStore.get_username(message.author_id) }})</span><span class="date">{{ new Date(message.timestamp).toLocaleTimeString() }}</span>
            </div>
            <div v-else>
                <span class="author">{{ userStore.get_username(message.author_id) }}</span><span class="date">{{ new Date(message.timestamp).toLocaleTimeString() }}</span>
            </div>
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

.message-history > .message > div > .author {
    font-weight: bold;
    font-size: larger;
}
.message-history > .message > div > .date {
    color: #222;
    font-size: smaller;
}
</style>
