import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import type { User } from '@/types';
import { api } from '@/lib/api/client';

export const useAuthStore = defineStore('auth', () => {
	const user = ref<User | null>(null);
	const isAuthenticated = computed(() => user.value != null);

	function setUser(value: User | null) {
		user.value = value;
	}

	async function logout() {
		try {
			await api.logout();
		} finally {
			user.value = null;
		}
	}

	return { user, isAuthenticated, setUser, logout };
});
