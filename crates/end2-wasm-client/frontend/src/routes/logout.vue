<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { request } from '../api'
import { useUserStore } from '../stores/user'

const query = useQueryClient()
const router = useRouter()
const user_store = useUserStore()

async function logout() {
    return await request<void>('/auth/logout', 'POST')
}

const { mutate } = useMutation({
    mutationFn: logout,
    onSuccess: () => {
        user_store.logout()
        query.clear()
        router.replace('/')
    },
    onError: (err) => {
        console.error('Logout failed', err)
        user_store.logout()
        query.clear()
        router.replace('/')
    },
})

onMounted(() => {
    mutate()
})
</script>

<template></template>
