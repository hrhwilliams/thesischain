<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router'
import { useQueryClient } from '@tanstack/vue-query'

import type { ApiError } from '../api';
import { login } from '../api';
import { useClientState } from '../state';

const username = ref('')
const password = ref('')
const error = ref<ApiError | null>(null)
const loading = ref(false)
const router = useRouter()
const query = useQueryClient()
const state = useClientState()

async function onSubmit() {
    error.value = null

    try {
        loading.value = true

        const response = await login({
            username: username.value,
            password: password.value,
        })

        if (response.ok) {
            state.login(response.data)
            query.setQueryData(['me'], response.data);
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
    <div class="login">
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
            autocomplete="password"
            required
            />
        </label>
        <br>
        <button type="submit" :disabled="loading">
            {{ loading ? 'Logging in' : 'Log in' }}
        </button>
    </form>
    <p v-if="error" class="error">
        <strong>Error response from server</strong><br>{{ error.message }}
        <span v-if="error.detail">: {{ error.detail }}</span>
    </p>
    </div>
</template>

<style>
</style>