<script setup lang="ts">
import { computed, watch } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import { useRoute } from 'vue-router'
import { me } from './api'
import { useClientState } from './state'

const route = useRoute()
const state = useClientState()
let pageTitle = computed(() => route.meta.title || 'End2')

const { data: user } = useQuery({
    queryKey: ['me'],
    queryFn: async () => {
        const response = await me()

        if (response.ok) {
            return response.data
        }
        return null
    },
    retry: false,
})

watch(user, async (new_user) => {
    if (!state.is_logged_in && new_user) {
        state.login(new_user)
    }

    if (state.is_logged_in) {
        if (!state.crypto) {
            await state.init_device()
        }

        if (state.crypto) {
            await state.upload_otks()

            if (!state.ws) {
                state.init_ws()
            }
        }
    }
})

// onUnmounted(() => disconnect_websocket())
</script>

<template>
    <div class="container">
        <header>
            <h1>{{ pageTitle }}</h1>
            <nav>
                <div>
                    <p v-if="state.user">
                        Logged in as <span v-if="state.user.nickname" ><strong>{{ state.user.nickname }}</strong> ({{ state.user.username }})</span>
                        <strong v-else>{{ state.user.username }}</strong>
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
        <router-view />
    </main>
</template>
