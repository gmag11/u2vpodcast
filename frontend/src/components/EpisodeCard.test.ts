import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import EpisodeCard from '@/components/EpisodeCard.vue';
import { testI18n } from '@/test/i18n';
import { usePlayerStore } from '@/stores/player';
import type { Episode } from '@/types';

function episode(overrides: Partial<Episode> = {}): Episode {
	const now = new Date();
	return {
		id: 1,
		channel_id: 1,
		channel_slug: 'c',
		channel_title: 'Channel',
		title: 'Episode 1',
		description: 'Description',
		yt_id: 'yt1',
		webpage_url: 'https://www.youtube.com/watch',
		published_at: now,
		duration: '01:00:00',
		image: '',
		listen: false,
		position_seconds: 0,
		listened_at: null,
		created_at: now,
		updated_at: now,
		...overrides
	};
}

function mountCard(ep: Episode) {
	return mount(EpisodeCard, {
		props: { episode: ep },
		global: { plugins: [createPinia(), testI18n], stubs: { RouterLink: true } }
	});
}

describe('EpisodeCard playback indicators', () => {
	beforeEach(() => {
		localStorage.clear();
	});

	it('shows the played mark on completed episodes', () => {
		const wrapper = mountCard(episode({ listen: true, position_seconds: 3600 }));
		expect(wrapper.find('[data-testid="listened-mark"]').exists()).toBe(true);
		expect(wrapper.find('[aria-label="Listened"]').exists()).toBe(true);
	});

	it('shows a resume hint for partially played episodes', () => {
		const wrapper = mountCard(episode({ position_seconds: 100 }));
		expect(wrapper.text()).toContain('Continue at 01:40');
	});

	it('shows neither indicator for untouched episodes', () => {
		const wrapper = mountCard(episode({ position_seconds: 0 }));
		expect(wrapper.find('[aria-label="Listened"]').exists()).toBe(false);
		expect(wrapper.text()).not.toContain('Continue at');
	});

	it('shows a read-only progress strip sized to the saved position', () => {
		const wrapper = mountCard(episode({ position_seconds: 1800 })); // 50% of 01:00:00
		const bar = wrapper.find('[data-testid="episode-progress"]');
		expect(bar.exists()).toBe(true);
		expect(bar.find('div').attributes('style')).toContain('width: 50%');
	});

	it('sizes the progress strip to 100% for completed episodes', () => {
		const wrapper = mountCard(episode({ listen: true, position_seconds: 3600 }));
		const bar = wrapper.find('[data-testid="episode-progress"]');
		expect(bar.exists()).toBe(true);
		expect(bar.find('div').attributes('style')).toContain('width: 100%');
	});

	it('enables stop on a non-current episode and resets its progress', async () => {
		const pinia = createPinia();
		setActivePinia(pinia);
		const player = usePlayerStore();
		const spy = vi.spyOn(player, 'stop').mockImplementation(() => undefined);
		const wrapper = mount(EpisodeCard, {
			props: { episode: episode({ position_seconds: 100 }) },
			global: { plugins: [pinia, testI18n], stubs: { RouterLink: true } }
		});

		const stopBtn = wrapper.find('[aria-label="Stop"]');
		expect((stopBtn.element as HTMLButtonElement).disabled).toBe(false);
		await stopBtn.trigger('click');
		expect(spy).toHaveBeenCalledWith(expect.objectContaining({ id: 1 }));
	});

	it('does not render the progress strip without a saved position', () => {
		const wrapper = mountCard(episode({ position_seconds: 0 }));
		expect(wrapper.find('[data-testid="episode-progress"]').exists()).toBe(false);
	});
});