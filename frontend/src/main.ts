import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from '@/router';
import i18n from '@/i18n';
import { useAuthStore } from '@/stores/auth';
import { useThemeStore } from '@/stores/theme';
import { useLocaleStore } from '@/stores/locale';
import { api } from '@/lib/api/client';
import '@/app.css';

const app = createApp(App);
app.use(createPinia());
app.use(i18n);

const theme = useThemeStore();
theme.init();

const locale = useLocaleStore();
locale.init();

async function bootstrap() {
	const auth = useAuthStore();
	if (!auth.isAuthenticated) {
		const result = await api.getSession();
		if (result.ok && result.user) {
			auth.setUser(result.user);
		}
	}
	app.use(router);
	app.mount('#app');
}

bootstrap();
