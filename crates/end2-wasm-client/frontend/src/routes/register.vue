<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useQueryClient } from '@tanstack/vue-query'
import { request, type ApiError } from '../api';
import ErrorMessage from '../components/ErrorMessage.vue';
import type { UserInfo } from '../types/user';
import { useUserStore } from '../stores/user';

const username = ref('')
const password = ref('')
const confirm_password = ref('')
const error = ref<ApiError | null>(null)
const loading = ref(false)
const router = useRouter()
const query = useQueryClient()
const user_store = useUserStore()

async function onSubmit() {
    error.value = null

    try {
        loading.value = true

        const response = await request<UserInfo>('/auth/register', 'POST', {
            username: username.value,
            password: password.value,
            confirm_password: confirm_password.value,
        })

        if (response.ok) {
            user_store.login(response.value)
            query.setQueryData(['me'], response.value);
            router.replace('/chats')
        } else {
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
        loading.value = false
    }
}
</script>

<template>
    <div class="register">
    <form @submit.prevent="onSubmit">
        <label>
            Username
            <input
            v-model="username"
            type="text"
            autocomplete="username"
            required
            />
        </label>
        <br>
        <label>
            Password
            <input
            v-model="password"
            type="password"
            autocomplete="new-password"
            required
            />
        </label>
        <br>
        <label>
            Confirm Password
            <input
            v-model="confirm_password"
            type="password"
            autocomplete="new-password"
            required
            />
        </label>
        <br>
        <button type="submit" :disabled="loading">
            {{ loading ? 'Registering' : 'Register' }}
        </button>
    </form>
    <ErrorMessage
        v-if="error"
        :status="error.status"
        :message="error.message"
        :detail="error.detail">
    </ErrorMessage>
    </div>
</template>
