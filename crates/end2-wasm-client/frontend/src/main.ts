import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { VueQueryPlugin } from '@tanstack/vue-query';
import './style.css'
import App from './App.vue'
import router from './router'

const pinia = createPinia()

const app = createApp(App)
    .use(pinia)
    .use(router)
    .use(VueQueryPlugin)

app.mount('#app')
    