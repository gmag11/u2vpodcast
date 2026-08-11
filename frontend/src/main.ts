import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from '@/router';
import { useAuthStore } from '@/stores/auth';
import { useThemeStore } from '@/stores/theme';
import { api } from '@/lib/api/client';
import '@/app.css';

const app = createApp(App);
app.use(createPinia());
app.use(router);

const theme = useThemeStore();
theme.init();

async function bootstrap() {
	const auth = useAuthStore();
	if (!auth.isAuthenticated) {
		const result = await api.getSession();
		if (result.ok && result.user) {
			auth.setUser(result.user);
		}
	}
	app.mount('#app');
}

bootstrap();
