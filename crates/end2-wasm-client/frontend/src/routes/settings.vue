<script setup lang="ts">
import { computed, ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import type { ApiError } from '../api'
import { useUserStore } from '../stores/user'
import { useDeviceStore } from '../stores/device'
import ErrorMessage from '../components/ErrorMessage.vue'

const error = ref<ApiError | null>(null)
const nickname = ref<string>('')
const user_store = useUserStore()
const device_store = useDeviceStore()
const loading = ref(false)

const { data: devices } = useQuery({
    queryKey: ['devices'],
    enabled: computed(() => user_store.logged_in),
    queryFn: async () => {
        const response = await device_store.fetch_our_devices()
        if (response.ok) {
            return response.value
        } else {
            error.value = response.error
            return null
        }
    },
})

async function onSubmit() {
    error.value = null

    try {
        loading.value = true

        const response = await user_store.change_nickname(nickname.value)

        if (!response.ok) {
            error.value = response.error
        }
    } catch (e) {
        console.error(e)
        error.value = {
            status: 0,
            message: 'Unexpected',
            detail: e instanceof Error ? e.message : undefined
        }
    } finally {
        nickname.value = ''
        loading.value = false
    }
}
</script>

<template>
    <div class="device-list">
        <div v-if="user_store.me">
            <h3>Your user ID</h3>
            <code>{{ user_store.me.id }}</code>
        </div>
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
            <li v-for="device in devices">
                <div v-if="device.device_id === device_store.device_id()"><code>{{ device.device_id }}</code> (this device)</div>
                <div v-else><code>{{ device.device_id }}</code></div>
            </li>
        </ul>
        <ErrorMessage
            v-if="error"
            :status="error.status"
            :message="error.message"
            :detail="error.detail">
        </ErrorMessage>
    </div>
</template>
