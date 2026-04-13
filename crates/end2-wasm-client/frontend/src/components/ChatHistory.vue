<script setup lang="ts">
import { nextTick, toRef, ref, watch } from "vue";
import { from, useObservable } from "@vueuse/rxjs";
import { liveQuery, Dexie } from "dexie";
import { db } from "../db";
import { useUserStore } from "../stores/user";
import { useMessageStore } from "../stores/message";
import { useChannelStore } from "../stores/channel";

const props = defineProps({
  channel_id: { type: String, required: true },
});

const isReady = ref(false);
const channel_id = toRef(props, "channel_id");
const userStore = useUserStore();
const channelStore = useChannelStore();
const messageStore = useMessageStore();
const history = ref<HTMLDivElement | null>(null);

const messages = useObservable(
  from(
    liveQuery(async () => {
      if (!channel_id.value) return [];

      return await db.messages
        .where("[channel_id+timestamp]")
        .between(
          [channel_id.value, Dexie.minKey],
          [channel_id.value, Dexie.maxKey],
        )
        .toArray();
    }),
  ),
);

watch(messages, async (new_msgs, old_msgs) => {
  const el = history.value;
  if (!el || !new_msgs) {
    return;
  }

  const is_at_bottom = el.scrollHeight - el.scrollTop - el.clientHeight <= 50;
  const is_first_load = !old_msgs || old_msgs.length === 0;

  await nextTick();

  if (is_first_load || is_at_bottom) {
    el.scrollTop = el.scrollHeight;
  }
});

watch(
  channel_id,
  async (channelId) => {
    if (!channelId) return;
    isReady.value = false;

    await messageStore.fetchHistory(channelId);

    const channelInfo = await channelStore.fetchChannel(channelId);
    if (channelInfo.ok) {
      channelInfo.value.participants.forEach((u) => userStore.updateUser(u));
    }

    isReady.value = true;
  },
  { immediate: true },
);
</script>

<template>
  <div v-if="isReady" ref="history" class="message-history">
    <div v-for="message in messages" :key="message.message_id" class="message">
      <div v-if="userStore.has_nickname(message.author_id)">
        <span class="author"
          >{{ userStore.get_display_name(message.author_id) }} ({{
            userStore.get_username(message.author_id)
          }})</span
        >
        <span class="date">{{
          new Date(message.timestamp).toLocaleTimeString()
        }}</span>
      </div>
      <div v-else>
        <span class="author">{{
          userStore.get_username(message.author_id)
        }}</span>
        <span class="date">{{
          new Date(message.timestamp).toLocaleTimeString()
        }}</span>
      </div>
      <div class="content">{{ message.plaintext }}</div>
    </div>
  </div>
  <div v-else>loading...</div>
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
