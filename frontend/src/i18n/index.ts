import { createI18n } from 'vue-i18n';
import en from './locales/en.json';
import es from './locales/es.json';

export const AVAILABLE_LOCALES = ['en', 'es'] as const;
export type Locale = (typeof AVAILABLE_LOCALES)[number];

const messages = {
	en,
	es
};

const datetimeFormats = {
	en: {
		short: { year: 'numeric', month: 'numeric', day: 'numeric' }
	},
	es: {
		short: { year: 'numeric', month: 'numeric', day: 'numeric' }
	}
} as const;

const i18n = createI18n({
	legacy: false,
	locale: 'en',
	fallbackLocale: 'en',
	availableLocales: [...AVAILABLE_LOCALES],
	datetimeFormats,
	messages
});

export default i18n;
