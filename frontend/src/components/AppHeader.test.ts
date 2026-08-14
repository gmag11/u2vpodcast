import { flushPromises, mount } from '@vue/test-utils';
import { createPinia } from 'pinia';
import { createMemoryHistory, createRouter } from 'vue-router';
import { beforeEach, describe, expect, it } from 'vitest';
import AppHeader from '@/components/AppHeader.vue';
import { useAuthStore } from '@/stores/auth';
import type { User } from '@/types';

const router = createRouter({
	history: createMemoryHistory(),
	routes: [
		{ path: '/', name: 'channels', component: { template: '<div />' } },
		{ path: '/history', name: 'history', component: { template: '<div />' } }
	]
});

const user: User = { id: 1, name: 'admin', role: 'Admin', active: true };

async function mountHeader() {
	const pinia = createPinia();
	useAuthStore(pinia).setUser(user);
	await router.push('/');
	await router.isReady();
	return mount(AppHeader, {
		global: { plugins: [router, pinia] },
		slots: { 'brand-icon': '<span class="brand-icon" />' }
	});
}

describe('AppHeader', () => {
	beforeEach(async () => {
		await router.push('/');
	});

	it('hides the wordmark and inline nav on mobile', async () => {
		const wrapper = await mountHeader();
		const wordmark = wrapper.findAll('span').find((el) => el.text() === 'U2VPodcast');
		expect(wordmark?.classes()).toContain('hidden');
		const inlineNav = wrapper
			.findAll('div')
			.find((el) => el.classes().includes('hidden') && el.classes().includes('md:flex'));
		expect(inlineNav).toBeTruthy();
	});

	it('opens the drawer with user, nav links and logout', async () => {
		const wrapper = await mountHeader();
		await wrapper.find('button[aria-label="Open menu"]').trigger('click');
		expect(wrapper.find('aside[aria-label="Navigation"]').exists()).toBe(true);
		expect(wrapper.text()).toContain('admin');
		expect(wrapper.text()).toContain('Channels');
		expect(wrapper.text()).toContain('History');
		expect(wrapper.text()).toContain('Logout');
	});

	it('closes the drawer when a nav link is selected', async () => {
		const wrapper = await mountHeader();
		await wrapper.find('button[aria-label="Open menu"]').trigger('click');
		await wrapper.find('aside a[href="/history"]').trigger('click');
		await flushPromises();
		expect(wrapper.find('aside[aria-label="Navigation"]').exists()).toBe(false);
	});

	it('closes the drawer on backdrop click', async () => {
		const wrapper = await mountHeader();
		await wrapper.find('button[aria-label="Open menu"]').trigger('click');
		const backdrop = wrapper.findAll('div').find((el) => el.classes().includes('z-[60]'));
		expect(backdrop).toBeTruthy();
		await backdrop!.trigger('click');
		expect(wrapper.find('aside[aria-label="Navigation"]').exists()).toBe(false);
	});

	it('closes the drawer on Escape', async () => {
		const wrapper = await mountHeader();
		await wrapper.find('button[aria-label="Open menu"]').trigger('click');
		expect(wrapper.find('aside[aria-label="Navigation"]').exists()).toBe(true);
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
		await flushPromises();
		expect(wrapper.find('aside[aria-label="Navigation"]').exists()).toBe(false);
	});
});
