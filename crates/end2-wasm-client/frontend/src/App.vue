<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useRoute } from 'vue-router'
import { useUserStore } from './stores/user'
import { request, type ApiError } from './api'
import type { UserInfo } from './types/user'
import { useDeviceStore } from './stores/device'
import { useWebSocketStore } from './stores/socket'
import ErrorMessage from './components/ErrorMessage.vue'

const route = useRoute()
let pageTitle = computed(() => route.meta.title || 'End2')

const user_store = useUserStore()
const device_store = useDeviceStore()
const socket = useWebSocketStore()

const error = ref<ApiError | null>(null)

const { } = useQuery({
    queryKey: ['me'],
    queryFn: async () => {
        const response = await request<UserInfo>('/me', 'GET')

        if (response.ok) {
            user_store.login(response.value)
            error.value = null
            return response.value
        } else {
            if (response.error.status != 401) {
                error.value = response.error
            }
            return null
        }
    },
    retry: false,
})

watch(
    () => user_store.me,
    async (me) => {
        if (me) {
            const response = await device_store.init(me)
            if (!response.ok) {
                error.value = response.error
            }

            const response2 = await device_store.otks()
            if (!response2.ok) {
                error.value = response2.error
            }

            socket.connect(device_store.device_id()!)
        } else {
            socket.disconnect()
        }
    }
)
</script>

<template>
    <div class="navbar">
        <header>
            <h1>{{ pageTitle }}</h1>
            <nav>
                <div>
                    <p v-if="user_store.logged_in && user_store.me">
                        Logged in as <span v-if="user_store.me.nickname"><strong>{{ user_store.me.nickname }}</strong> ({{ user_store.me.username }})</span>
                        <strong v-else>{{ user_store.me!.username }}</strong>
                        | <RouterLink to="/">Home</RouterLink>
                        | <RouterLink to="/chats">Chats</RouterLink>
                        | <RouterLink to="/settings">Settings</RouterLink>
                        | <RouterLink to="/logout">Log Out</RouterLink></p>
                    <p v-else>
                        <RouterLink to="/">Home</RouterLink>
                        | <RouterLink to="/login">Log In</RouterLink>
                        | <a href="http://localhost:8081/api/auth/discord">Log in with Discord</a>
                        | <RouterLink to="/register">Register</RouterLink>
                    </p>
                </div>
            </nav>
        </header>
        <hr>
    </div>
    <main>
        <router-view></router-view>
        <ErrorMessage
            v-if="error"
            :status="error.status"
            :message="error.message"
            :detail="error.detail">
        </ErrorMessage>
    </main>
</template>
