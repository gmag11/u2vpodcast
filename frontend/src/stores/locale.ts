import { ref } from 'vue';
import { defineStore } from 'pinia';
import i18n, { AVAILABLE_LOCALES, type Locale } from '@/i18n';

const LOCALE_KEY = 'locale';

function normalize(input: string): Locale {
	const lang = input.split('-')[0].toLowerCase();
	return (AVAILABLE_LOCALES as readonly string[]).includes(lang) ? (lang as Locale) : 'en';
}

export const useLocaleStore = defineStore('locale', () => {
	const locale = ref<Locale>('en');

	function resolveInitial(): Locale {
		const saved = localStorage.getItem(LOCALE_KEY);
		if (saved && (AVAILABLE_LOCALES as readonly string[]).includes(saved)) {
			return saved as Locale;
		}
		return normalize(navigator.language);
	}

	function apply(value: Locale) {
		locale.value = value;
		i18n.global.locale.value = value;
		document.documentElement.lang = value;
		localStorage.setItem(LOCALE_KEY, value);
	}

	function init() {
		apply(resolveInitial());
	}

	return { locale, init, apply };
});
