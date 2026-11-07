import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ShareView from '@/views/ShareView.vue';
import { testI18n } from '@/test/i18n';
import { api } from '@/lib/api/client';

vi.mock('vue-router', () => ({ useRoute: () => ({ params: { token: 'tok' } }) }));

vi.mock('@/lib/api/client', () => ({ api: { getSharedEpisode: vi.fn() } }));

function mountView() {
	return mount(ShareView, { global: { plugins: [testI18n] } });
}

describe('ShareView', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		testI18n.global.locale.value = 'en';
	});

	it('renders the shared episode metadata and a player', async () => {
		vi.mocked(api.getSharedEpisode).mockResolvedValue({
			title: 'Episode',
			channel_title: 'Channel',
			description: 'Description',
			image: 'https://images.example.com/cover.jpg',
			duration: '00:10:00',
			published_at: '2026-01-01T00:00:00Z',
			audio_url: '/s/tok/audio.mp3'
		});
		const wrapper = mountView();
		await flushPromises();
		expect(wrapper.find('[data-testid="share-episode"]').exists()).toBe(true);
		expect(wrapper.get('[data-testid="share-title"]').text()).toBe('Episode');
		expect(wrapper.get('[data-testid="share-channel"]').text()).toBe('Channel');
		expect(wrapper.get('[data-testid="share-audio"]').attributes('src')).toBe('/s/tok/audio.mp3');
		expect(api.getSharedEpisode).toHaveBeenCalledWith('tok');
	});

	it('shows the unavailable state when the token cannot be resolved', async () => {
		vi.mocked(api.getSharedEpisode).mockResolvedValue(null);
		const wrapper = mountView();
		await flushPromises();
		expect(wrapper.find('[data-testid="share-unavailable"]').exists()).toBe(true);
		expect(wrapper.find('[data-testid="share-audio"]').exists()).toBe(false);
	});
});
