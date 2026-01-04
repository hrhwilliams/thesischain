<script setup>
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { useAuth } from './common.js';

const route = useRoute();
const pageTitle = computed(() => route.meta.title || '');
const { user } = useAuth();
</script>

<template>
    <div class="container">
        <header>
            <h1>{{ pageTitle }}</h1>
            <nav>
                <div v-if="user">
                    <p>Logged in as {{  user.username  }} | <a href="/">Home</a> | <a href="/logout">Log Out</a></p>
                </div>
                <div v-else>
                    <p><a href="/">Home</a> | <a href="/login">Log In</a> | <a href="/register">Register</a></p>
                </div>
            </nav>
        </header>
        <hr>
        <router-view></router-view>
    </div>
</template>
