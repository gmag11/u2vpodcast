import { ref } from 'vue';
import { defineStore } from 'pinia';

export type Theme = 'light' | 'dark';

const THEME_KEY = 'theme';

export const useThemeStore = defineStore('theme', () => {
	const theme = ref<Theme>('dark');

	function resolveInitial(): Theme {
		const saved = localStorage.getItem(THEME_KEY);
		if (saved === 'light' || saved === 'dark') return saved;
		return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
	}

	function apply(value: Theme) {
		theme.value = value;
		document.documentElement.classList.toggle('dark', value === 'dark');
		localStorage.setItem(THEME_KEY, value);
	}

	function init() {
		apply(resolveInitial());
	}

	function toggle() {
		apply(theme.value === 'dark' ? 'light' : 'dark');
	}

	return { theme, init, apply, toggle };
});
