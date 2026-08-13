import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '@/stores/auth';

const router = createRouter({
	history: createWebHistory('/app'),
	routes: [
		{
			path: '/login',
			name: 'login',
			component: () => import('@/views/LoginView.vue'),
			meta: { public: true }
		},
		{
			path: '/',
			name: 'channels',
			component: () => import('@/views/ChannelsView.vue')
		},
		{
			path: '/:channelId(\\d+)',
			name: 'episodes',
			component: () => import('@/views/EpisodesView.vue')
		}
	]
});

router.beforeEach((to) => {
	const auth = useAuthStore();

	if (to.meta.public && auth.isAuthenticated) {
		return { name: 'channels' };
	}

	if (!to.meta.public && !auth.isAuthenticated) {
		return {
			name: 'login',
			query: { next: to.fullPath }
		};
	}

	return true;
});

export default router;
