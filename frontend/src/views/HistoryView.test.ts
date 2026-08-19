import { flushPromises, mount } from '@vue/test-utils';
import { createPinia } from 'pinia';
import { createMemoryHistory, createRouter } from 'vue-router';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import HistoryView from '@/views/HistoryView.vue';
import { api } from '@/lib/api/client';
import type { Episode } from '@/types';

const router = createRouter({
	history: createMemoryHistory(),
	routes: [
		{ path: '/', name: 'channels', component: { template: '<div />' } },
		{ path: '/history', name: 'history', component: HistoryView },
		{ path: '/:channelId(\\d+)', name: 'episodes', component: { template: '<div />' } }
	]
});

function episode(id: number, title: string, channelTitle = ''): Episode {
	const now = new Date();
	return {
		id,
		channel_id: 1,
		channel_slug: 'c',
		channel_title: channelTitle,
		title,
		description: 'Description',
		yt_id: `yt${id}`,
		webpage_url: 'https://www.youtube.com/watch',
		published_at: now,
		duration: '00:10:00',
		image: '',
		listen: false,
		created_at: now,
		updated_at: now
	};
}

function okResult(data: Episode[]) {
	return {
		ok: true,
		data,
		user: { id: 1, name: 'admin', role: 'Admin', active: true },
		status: true
	};
}

async function mountView() {
	await router.push('/history');
	await router.isReady();
	const wrapper = mount(HistoryView, {
		global: { plugins: [router, createPinia()] }
	});
	await flushPromises();
	return wrapper;
}

describe('HistoryView', () => {
	beforeEach(() => {
		vi.restoreAllMocks();
		vi.spyOn(api, 'getAllEpisodes');
	});

	it('renders episodes from the api and shows the channel name on each card', async () => {
		vi.mocked(api.getAllEpisodes).mockResolvedValue(
			okResult([episode(2, 'Episodio 42', 'Linux y Tapas'), episode(1, 'Episodio 10')]) as never
		);
		const wrapper = await mountView();
		expect(wrapper.text()).toContain('Episodio 42');
		expect(wrapper.text()).toContain('Episodio 10');
		expect(wrapper.text()).toContain('Linux y Tapas');
		expect(wrapper.text()).toContain('Description');
		expect(wrapper.find('a[href="/history"]').exists()).toBe(true);
		expect(wrapper.find('a[href="/1"]').text()).toContain('Linux y Tapas');
	});

	it('shows an empty state when there are no episodes', async () => {
		vi.mocked(api.getAllEpisodes).mockResolvedValue(okResult([]) as never);
		const wrapper = await mountView();
		expect(wrapper.text()).toContain('No episodes yet');
	});

	it('lays out cards in a single wide column, not a grid', async () => {
		vi.mocked(api.getAllEpisodes).mockResolvedValue(
			okResult([episode(1, 'Episodio 10'), episode(2, 'Episodio 42')]) as never
		);
		const wrapper = await mountView();
		const mainHtml = wrapper.find('main').html();
		expect(mainHtml).toContain('flex-col');
		expect(mainHtml).not.toContain('grid-cols');
		expect(mainHtml).not.toContain('lg:grid');
	});

	it('filters cards live as the user types', async () => {
		vi.mocked(api.getAllEpisodes).mockResolvedValue(
			okResult([episode(2, 'Episodio 42'), episode(1, 'Episodio 10')]) as never
		);
		const wrapper = await mountView();
		const input = wrapper.find('input[placeholder="Search episodes…"]');
		await input.setValue('42');
		expect(wrapper.text()).toContain('Episodio 42');
		expect(wrapper.text()).not.toContain('Episodio 10');
	});

	it('matches by yt_id and by multiple words in any order', async () => {
		vi.mocked(api.getAllEpisodes).mockResolvedValue(
			okResult([episode(7, 'Linux y Tapas'), episode(1, 'Episodio 10')]) as never
		);
		const wrapper = await mountView();
		const input = wrapper.find('input[placeholder="Search episodes…"]');
		await input.setValue('yt7');
		expect(wrapper.text()).toContain('Linux y Tapas');
		expect(wrapper.text()).not.toContain('Episodio 10');
		await input.setValue('tapas linux');
		expect(wrapper.text()).toContain('Linux y Tapas');
	});

	it('shows a no-results message for unmatched queries', async () => {
		vi.mocked(api.getAllEpisodes).mockResolvedValue(okResult([episode(1, 'Episodio 10')]) as never);
		const wrapper = await mountView();
		await wrapper.find('input[placeholder="Search episodes…"]').setValue('zzz');
		expect(wrapper.text()).toContain('No results match your search.');
	});

	it('clearing the search restores the full list', async () => {
		vi.mocked(api.getAllEpisodes).mockResolvedValue(
			okResult([episode(2, 'Episodio 42'), episode(1, 'Episodio 10')]) as never
		);
		const wrapper = await mountView();
		const input = wrapper.find('input[placeholder="Search episodes…"]');
		await input.setValue('42');
		expect(wrapper.text()).not.toContain('Episodio 10');
		await input.setValue('');
		expect(wrapper.text()).toContain('Episodio 10');
		expect(wrapper.text()).toContain('Episodio 42');
	});
});
