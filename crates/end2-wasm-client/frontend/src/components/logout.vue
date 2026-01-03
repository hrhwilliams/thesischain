<script setup>
import { onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuth } from '../store.js';

const router = useRouter();
const { logout } = useAuth();

onMounted(async () => {
    try {
        await logout();
    } catch (e) {
        console.error("Logout failed", e);
    } finally {
        router.replace('/'); 
    }
});
</script>

<template></template>