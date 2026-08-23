import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { createMemoryHistory, createRouter } from 'vue-router';
import { beforeEach, describe, expect, it } from 'vitest';
import App from '@/App.vue';
import PersistentPlayer from '@/components/PersistentPlayer.vue';
import { useAuthStore } from '@/stores/auth';
import { usePlayerStore } from '@/stores/player';
import { testI18n } from '@/test/i18n';
import type { User } from '@/types';

const router = createRouter({
	history: createMemoryHistory(),
	routes: [{ path: '/', name: 'channels', component: { template: '<div />' } }]
});

const admin: User = { id: 1, name: 'admin', role: 'Admin', active: true };

describe('App player gating', () => {
	beforeEach(() => {
		localStorage.clear();
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
});