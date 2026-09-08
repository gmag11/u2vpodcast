import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, describe, expect, it, vi } from 'vitest';
import ScrollingText from '@/components/ScrollingText.vue';

function mockMeasurements(viewportWidth: number, textWidth: number) {
	const clientWidth = vi
		.spyOn(HTMLElement.prototype, 'clientWidth', 'get')
		.mockImplementation(function (this: HTMLElement) {
			return this.dataset.testid === 'scrolling-text-viewport' ? viewportWidth : 0;
		});
	const scrollWidth = vi
		.spyOn(HTMLElement.prototype, 'scrollWidth', 'get')
		.mockImplementation(function (this: HTMLElement) {
			return this.dataset.testid === 'scrolling-text-text' ? textWidth : 0;
		});
	return () => {
		clientWidth.mockRestore();
		scrollWidth.mockRestore();
	};
}

function mockReducedMotion(matches: boolean) {
	vi.stubGlobal(
		'matchMedia',
		vi.fn(() => ({ matches }))
	);
}

afterEach(() => {
	vi.restoreAllMocks();
	vi.unstubAllGlobals();
});

describe('ScrollingText', () => {
	it('scrolls when active and overflowing', async () => {
		const restoreMeasurements = mockMeasurements(100, 200);
		mockReducedMotion(false);
		const wrapper = mount(ScrollingText, { props: { text: 'A long title', active: true } });
		await flushPromises();

		const track = wrapper.get('[data-testid="scrolling-text-track"]');
		expect(track.classes()).toContain('scrolling-text-track--active');
		expect(track.attributes('style')).toContain('--scrolling-text-distance: 232px');
		expect(track.attributes('style')).toContain('--scrolling-text-duration: 7.25s');
		expect(wrapper.get('[data-testid="scrolling-text-text"]').classes()).toContain('inline-block');
		expect(wrapper.get('[data-testid="scrolling-text-copy"]').attributes('aria-hidden')).toBe(
			'true'
		);
		restoreMeasurements();
	});

	it('stays static when the text fits', async () => {
		const restoreMeasurements = mockMeasurements(200, 100);
		mockReducedMotion(false);
		const wrapper = mount(ScrollingText, { props: { text: 'Short title', active: true } });
		await flushPromises();

		const track = wrapper.get('[data-testid="scrolling-text-track"]');
		expect(track.classes()).toContain('truncate');
		expect(track.classes()).not.toContain('scrolling-text-track--active');
		expect(wrapper.find('[data-testid="scrolling-text-copy"]').exists()).toBe(false);
		restoreMeasurements();
	});

	it('stays static when inactive', async () => {
		const restoreMeasurements = mockMeasurements(100, 200);
		mockReducedMotion(false);
		const wrapper = mount(ScrollingText, { props: { text: 'A long title', active: false } });
		await flushPromises();

		const track = wrapper.get('[data-testid="scrolling-text-track"]');
		expect(track.classes()).toContain('truncate');
		expect(track.classes()).not.toContain('scrolling-text-track--active');
		restoreMeasurements();
	});

	it('does not animate when reduced motion is requested', async () => {
		const restoreMeasurements = mockMeasurements(100, 200);
		mockReducedMotion(true);
		const wrapper = mount(ScrollingText, { props: { text: 'A long title', active: true } });
		await flushPromises();

		const track = wrapper.get('[data-testid="scrolling-text-track"]');
		expect(track.classes()).toContain('truncate');
		expect(track.classes()).not.toContain('scrolling-text-track--active');
		restoreMeasurements();
	});
});
