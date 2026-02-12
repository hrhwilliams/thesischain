<script setup lang="ts">
import { computed, ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import type { ApiError } from '../api'
import { useUserStore } from '../stores/user'
import { useDeviceStore } from '../stores/device'
import { getDeviceId } from '../services/crypto'
import ErrorMessage from '../components/ErrorMessage.vue'

const error = ref<ApiError | null>(null)
const nickname = ref<string>('')
const userStore = useUserStore()
const deviceStore = useDeviceStore()
const loading = ref(false)

const { data: devices } = useQuery({
    queryKey: ['devices'],
    enabled: computed(() => userStore.logged_in),
    queryFn: async () => {
        const response = await deviceStore.fetchOurDevices()
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

        const response = await userStore.change_nickname(nickname.value)

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
        <div v-if="userStore.me">
            <h3>Your user ID</h3>
            <code>{{ userStore.me.id }}</code>
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
                <div v-if="device.device_id === getDeviceId()"><code>{{ device.device_id }}</code> (this device)</div>
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
    <h3>Backend version</h3>
    <code>7bb8925435d40b1010e028c2c62ede32a8c8effd</code>
</template>
