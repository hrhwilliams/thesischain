<script setup>
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuth } from '../common.js';

const router = useRouter();
const { login, isLoggingIn, loginError } = useAuth();
const usernameInput = ref("");

const handleLogin = async () => {
    if (!usernameInput.value) return;

    try {
        await login(usernameInput.value);
        router.push('/');
    } catch (e) {
        console.error("Login flow failed:", e);
    }
};
</script>

<template>
    <div class="login-container">
        <form @submit.prevent="handleLogin">
            <p>
                <label for="username">Username</label><br>
                <input 
                    type="text" 
                    id="username" 
                    name="username" 
                    v-model="usernameInput" 
                    required 
                    :disabled="isLoggingIn"
                    placeholder="Enter your username"
                >
            </p>

            <p v-if="loginError" class="error">
                Error: {{ loginError.message }}
            </p>

            <input 
                type="submit" 
                :value="isLoggingIn ? 'Verifying Keys...' : 'Log in'"
                :disabled="isLoggingIn"
            >
        </form>
    </div>
</template>