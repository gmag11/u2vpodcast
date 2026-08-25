import { mount } from '@vue/test-utils';
import { createPinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import EpisodeCard from '@/components/EpisodeCard.vue';
import { testI18n } from '@/test/i18n';
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
		expect(wrapper.text()).toContain('Listened');
	});

	it('shows a resume hint for partially played episodes', () => {
		const wrapper = mountCard(episode({ position_seconds: 100 }));
		expect(wrapper.text()).toContain('Continue at 01:40');
	});

	it('shows neither indicator for untouched episodes', () => {
		const wrapper = mountCard(episode({ position_seconds: 0 }));
		expect(wrapper.text()).not.toContain('Listened');
		expect(wrapper.text()).not.toContain('Continue at');
	});
});