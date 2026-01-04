import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { authService } from './auth.js';

export function useAuth() {
    const queryClient = useQueryClient();

    const { 
        data: user, 
        isLoading: isUserLoading, 
        error: userError 
    } = useQuery({
        queryKey: ['user'],
        queryFn: async () => {
            try {
                return await authService.getMe();
            } catch (err) {
                if (err.message.includes('401') || err.message.includes('403')) return null;
                throw err;
            }
        },
        retry: false,
        refetchOnWindowFocus: false,
    });

    const { 
        mutateAsync: login, 
        isPending: isLoggingIn, 
        error: loginError 
    } = useMutation({
        mutationFn: (username) => authService.login(username),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['user'] });
        },
    });

    const { 
        mutateAsync: register, 
        isPending: isRegistering, 
        error: registerError 
    } = useMutation({
        mutationFn: (username) => authService.register(username),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['user'] });
        },
    });

    return {
        user,
        isLoading: isUserLoading,
        isLoggingIn,
        isRegistering,
        login,
        register,
        loginError,
        registerError,
        userError
    };
}