import { fileURLToPath, URL } from 'node:url';

import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	base: '/app',
	plugins: [vue(), tailwindcss()],
	resolve: {
		alias: {
			'@': fileURLToPath(new URL('./src', import.meta.url))
		}
	},
	server: {
		proxy: {
			'/api': 'http://localhost:6996',
			'/media': 'http://localhost:6996',
			'/channels': 'http://localhost:6996'
		}
	}
});
