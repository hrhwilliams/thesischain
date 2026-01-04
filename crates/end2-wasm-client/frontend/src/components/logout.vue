<script setup>
import { onMounted } from 'vue';
import { useMutation, useQueryClient } from '@tanstack/vue-query';
import { useRouter } from 'vue-router';
import { authService } from '../auth.js';

const router = useRouter();
const queryClient = useQueryClient();

const { mutateAsync: logout } = useMutation({
    mutationFn: async () => {
        await authService.logout();
    },
    onSuccess: () => {
        queryClient.setQueryData(['user'], null);
        queryClient.invalidateQueries({ queryKey: ['user'] });
    },
    onError: (err) => {
        console.error("Logout failed on server:", err);
        queryClient.setQueryData(['user'], null);
    }
});

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