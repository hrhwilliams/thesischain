<script setup>
import { computed, onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import { useAuth } from '../common.js';
import { authService } from '../auth.js'

const { user } = useAuth();

const route = useRoute();

onMounted(async () => {
    if (!authService.channel_has_session(route.params.id)) {
        await authService.create_session_in_channel(route.params.id);
    } else {
        console.log("found existing session");
    }
})
</script>

<template>{{ $route.params.id }}</template>