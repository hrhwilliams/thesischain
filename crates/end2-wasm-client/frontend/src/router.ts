import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

import chat from "./routes/chat.vue"
import chats from "./routes/chats.vue"
import home from "./routes/home.vue"
import login from "./routes/login.vue"
import logout from "./routes/logout.vue"
import register from "./routes/register.vue"
import settings from "./routes/settings.vue"

const routes: RouteRecordRaw[] = [
    { path: '/', component: home, meta: { 'title': 'Home' } },
    { path: '/chats', component: chats, meta: { 'title': 'Chats' } },
    { path: '/chat/:chat_id', component: chat, meta: { 'title': 'Chat' } },
    { path: '/login', component: login, meta: { 'title': 'Login' } },
    { path: '/logout', component: logout },
    { path: '/register', component: register, meta: { 'title': 'Register' } },
    { path: '/settings', component: settings, meta: { 'title': 'Settings' } },
]

const router = createRouter({
    history: createWebHistory(),
    routes
})

export default router
