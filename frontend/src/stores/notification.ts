import { ref } from 'vue';
import { defineStore } from 'pinia';

export interface AppNotification {
	message: string;
	type: 'success' | 'error' | 'info';
}

export const useNotificationStore = defineStore('notification', () => {
	const current = ref<AppNotification | null>(null);
	let timeout: ReturnType<typeof setTimeout> | null = null;

	function show(message: string, type: AppNotification['type'] = 'info', duration = 3500) {
		current.value = { message, type };
		if (timeout) clearTimeout(timeout);
		timeout = setTimeout(() => {
			current.value = null;
		}, duration);
	}

	function clear() {
		current.value = null;
		if (timeout) clearTimeout(timeout);
	}

	return { current, show, clear };
});
