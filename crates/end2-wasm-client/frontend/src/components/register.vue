<script setup>
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuth } from '../store.js';

const router = useRouter();
const { register, isRegistering, registerError } = useAuth();
const usernameInput = ref("");

const handleRegister = async () => {
    if (!usernameInput.value) return;

    try {
        await register(usernameInput.value);
        router.push('/');
    } catch (e) {
        console.error("Registration flow failed:", e);
    }
};
</script>

<template>
    <div class="register-container">
        <form @submit.prevent="handleRegister">
            <p>
                <label for="username">Username</label><br>
                <input 
                    type="text" 
                    id="username" 
                    name="username" 
                    v-model="usernameInput" 
                    required 
                    :disabled="isRegistering"
                    placeholder="Enter your username"
                >
            </p>

            <p v-if="registerError" class="error">
                Error: {{ registerError.message }}
            </p>

            <input 
                type="submit" 
                :value="isRegistering ? 'Verifying Keys...' : 'Register'"
                :disabled="isRegistering"
            >
        </form>
    </div>
</template>