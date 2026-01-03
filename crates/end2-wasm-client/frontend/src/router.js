import { createRouter, createWebHistory } from 'vue-router';
import dashboard from './components/dashboard.vue'
import login from './components/login.vue'
import logout from './components/logout.vue'
import register from './components/register.vue'

const routes = [
    { path: '/', component: dashboard, meta: { title: 'Dashboard' } },
    { path: '/login', component: login, meta: { title: 'Login' } },
    { path: '/logout', component: logout },
    { path: '/register', component: register, meta: { title: 'Register' } },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;