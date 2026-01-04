<script setup>
import { computed, ref } from 'vue';
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { authService } from '../auth.js'
import { useAuth } from '../common.js';

const { user } = useAuth();
const queryClient = useQueryClient();
const newReceiver = ref('');

const { 
    data: channels, 
} = useQuery({
    queryKey: ['channels'],
    queryFn: async () => {
        try {
            return await authService.getChannels();
        } catch (err) {
            if (err.message.includes('401') || err.message.includes('403')) return null;
            throw err;
        }
    },
    enabled: computed(() => !!user.value),
    retry: false,
    refetchOnWindowFocus: false,
});

const { 
    mutateAsync: createChannel, 
    isPending: isCreating, 
    error: createError 
} = useMutation({
    mutationFn: (username) => authService.createChannel(username),
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ['channels'] });
        newReceiver.value = '';
    }
});

const handleCreate = async () => {
    if (newReceiver.value)
        await createChannel(newReceiver.value);
};
</script>

<template>
    <div v-if="channels">
        <div class="create-section">
            <form @submit.prevent="handleCreate">
                <input 
                    v-model="newReceiver" 
                    placeholder="Enter username to message..." 
                    :disabled="isCreating"
                />
                <button type="submit" :disabled="isCreating || !newReceiver">
                    {{ isCreating ? 'Creating...' : 'Start Chat' }}
                </button>
            </form>
            <p v-if="createError" class="error">
                {{ createError.message }}
            </p>
        </div>
        <ul v-if="channels && channels.length > 0">
        <li v-for="channel in channels" :key="channel.id">
            <router-link :to="`/channel/${channel.id}`" class="channel-link">
            <span class="name">{{ channel.sender }} -> {{ channel.receiver }}</span>
            </router-link>
        </li>
    </ul>
    </div>
</template>