<script setup lang="ts">
import { computed, ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useClientState } from '../state'
import { get_devices } from '../api'

const nickname = ref<string>('')
const state = useClientState()
const loading = ref(false)

const { data: devices } = useQuery({
    queryKey: ['devices'],
    enabled: computed(() => state.is_logged_in),
    queryFn: async () => {
        const response = await get_devices()
        if (!response.ok) {
            throw response.error
        }
        return response.data
    },
})

async function onSubmit() {
    nickname.value = ''
}
</script>

<template>
    <div class="device-list">
        <h3>Your user ID</h3>
        <code>{{ state.user?.id }}</code>
        <h3>Change nickname</h3>
        <form @submit.prevent="onSubmit">
            <label>
                Nickname
                <input
                v-model="nickname"
                type="text"
                autocomplete="username"
                required
                />
            </label>
            <br>
            <button type="submit" :disabled="loading">
                {{ loading ? 'Submitting' : 'Submit' }}
            </button>
        </form>
        <h3>Connected devices</h3>
        <ul>
            <li v-for="device in devices" :key="device.id">
                <div v-if="device.id === state.crypto?.device_id()"><code>{{ device.id }}</code> (this device)</div>
                <div v-else><code>{{ device.id }}</code></div>
            </li>
        </ul>
    </div>
</template>