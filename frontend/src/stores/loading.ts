import { computed, ref } from 'vue';
import { defineStore } from 'pinia';

export type LoadingStatus = 'IDLE' | 'LOADING';

export const useLoadingStore = defineStore('loading', () => {
	const status = ref<LoadingStatus>('IDLE');
	const message = ref('');
	const isLoading = computed(() => status.value === 'LOADING');

	function start(msg = '') {
		status.value = 'LOADING';
		message.value = msg;
	}

	function stop() {
		status.value = 'IDLE';
		message.value = '';
	}

	return { status, message, isLoading, start, stop };
});
