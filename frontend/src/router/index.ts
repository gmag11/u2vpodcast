import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '@/stores/auth';

const router = createRouter({
	history: createWebHistory('/app'),
	routes: [
		{
			path: '/login',
			name: 'login',
			component: () => import('@/views/LoginView.vue'),
			meta: { guestOnly: true }
		},
		{
			path: '/',
			name: 'channels',
			component: () => import('@/views/ChannelsView.vue')
		},
		{
			path: '/history',
			name: 'history',
			component: () => import('@/views/HistoryView.vue')
		},
		{
			path: '/playlist',
			name: 'playlist',
			component: () => import('@/views/PlaylistView.vue')
		},
		{
			path: '/:channelId(\\d+)',
			name: 'episodes',
			component: () => import('@/views/EpisodesView.vue')
		},
		{
			// Public share page: reachable without a session, and an already
			// authenticated visitor is not redirected away from it.
			path: '/share/:token',
			name: 'share',
			component: () => import('@/views/ShareView.vue'),
			meta: { public: true }
		}
	]
});

router.beforeEach((to) => {
	const auth = useAuthStore();

	if (to.meta.guestOnly && auth.isAuthenticated) {
		return { name: 'channels' };
	}

	if (!to.meta.public && !to.meta.guestOnly && !auth.isAuthenticated) {
		return {
			name: 'login',
			query: { next: to.fullPath }
		};
	}

	return true;
});

export default router;
