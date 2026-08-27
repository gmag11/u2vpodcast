import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { createMemoryHistory, createRouter } from 'vue-router';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import App from '@/App.vue';
import PersistentPlayer from '@/components/PersistentPlayer.vue';
import { useAuthStore } from '@/stores/auth';
import { usePlayerStore } from '@/stores/player';
import { usePlaylistStore } from '@/stores/playlists';
import { api } from '@/lib/api/client';
import { testI18n } from '@/test/i18n';
import type { Episode, User } from '@/types';

const router = createRouter({
	history: createMemoryHistory(),
	routes: [{ path: '/', name: 'channels', component: { template: '<div />' } }]
});

const admin: User = { id: 1, name: 'admin', role: 'Admin', active: true };

function episode(id: number): Episode {
	const now = new Date();
	return {
		id,
		channel_id: 1,
		channel_slug: 'c',
		channel_title: 'Canal',
		title: `Episodio ${id}`,
		description: 'Description',
		yt_id: `yt${id}`,
		webpage_url: 'https://www.youtube.com/watch',
		published_at: now,
		duration: '00:10:00',
		image: '',
		listen: false,
		position_seconds: 0,
		listened_at: null,
		created_at: now,
		updated_at: now
	};
}

describe('App player gating', () => {
	beforeEach(() => {
		localStorage.clear();
		vi.restoreAllMocks();
		setActivePinia(createPinia());
	});

	async function mountApp() {
		await router.push('/');
		await router.isReady();
		return mount(App, { global: { plugins: [router, testI18n] } });
	}

	it('does not mount the player bar without an authenticated session', async () => {
		const auth = useAuthStore();
		auth.setUser(null);
		const wrapper = await mountApp();
		expect(wrapper.findComponent(PersistentPlayer).exists()).toBe(false);
	});

	it('mounts the player bar once a user is set', async () => {
		const auth = useAuthStore();
		auth.setUser(admin); // set before mount so the initial render already has a session
		const wrapper = await mountApp();
		expect(wrapper.findComponent(PersistentPlayer).exists()).toBe(true);
	});

	it('stops playback when the session is lost', async () => {
		const auth = useAuthStore();
		const player = usePlayerStore();
		auth.setUser(admin);
		// simulate active playback state
		player.currentTime = 30;
		player.stopped = false;

		const wrapper = await mountApp();
		auth.setUser(null);
		await wrapper.vm.$nextTick();
		expect(player.stopped).toBe(true);
		expect(player.playing).toBe(false);
		expect(player.currentTime).toBe(0);
		expect(wrapper.find('.fixed.bottom-0').exists()).toBe(false);
	});

	it('loads the playlist when the session is restored before mount', async () => {
		// main.ts restores the session before mounting, so at mount time the
		// auth store is already populated; the playlist must still load so the
		// episode-card icons are marked without visiting the playlist page.
		vi.spyOn(api, 'getPlaylist').mockResolvedValue({
			ok: true,
			user: admin,
			status: true,
			data: [episode(42)]
		} as never);
		const auth = useAuthStore();
		auth.setUser(admin); // set before mount, like the production bootstrap
		await mountApp();
		await flushPromises();
		const playlists = usePlaylistStore();
		expect(playlists.loaded).toBe(true);
		expect(playlists.episodeIdSet.has(42)).toBe(true);
	});
});
