<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { useClientState } from '../state'
import { logout } from '../api'

const query = useQueryClient()
const router = useRouter()
const state = useClientState()

const { mutate } = useMutation({
    mutationFn: logout,
    onSuccess: () => {
        query.setQueryData(['me'], null);
        state.logout()
        query.removeQueries();
        router.replace('/')
    }
});

onMounted(() => {
    mutate();
});
</script>

<template></template>