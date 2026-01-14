<script setup lang="ts">
const props = defineProps({
    status: { type: Number, required: true },
    message: { type: String, required: true },
    detail: String
})
</script>

<template>
    <p><strong>Error: </strong>{{ message }}<span v-if="detail">: {{ detail }}</span>.</p>
</template>