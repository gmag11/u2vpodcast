import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import ProgressScrubber from '@/components/ProgressScrubber.vue';

function mountScrubber(props = {}) {
	return mount(ProgressScrubber, {
		props: {
			progress: 50,
			duration: 600,
			...props
		}
	});
}

function mockRect(wrapper: ReturnType<typeof mount>) {
	vi.spyOn(wrapper.element, 'getBoundingClientRect').mockReturnValue({
		left: 0,
		width: 100,
		top: 0,
		height: 20,
		right: 100,
		bottom: 20,
		x: 0,
		y: 0,
		toJSON: () => ({})
	} as DOMRect);
	return wrapper;
}

function dispatchPointer(
	wrapper: ReturnType<typeof mount>,
	type: string,
	options: { clientX: number; pointerId: number }
) {
	const event = new MouseEvent(type, { bubbles: true, clientX: options.clientX });
	Object.defineProperty(event, 'pointerId', { value: options.pointerId });
	wrapper.element.dispatchEvent(event);
}

describe('ProgressScrubber', () => {
	it('renders a draggable thumb at the current playback position', () => {
		const w = mountScrubber({ progress: 40 });
		const thumb = w.get('[data-testid="player-progress-thumb"]');
		expect(thumb.attributes('style')).toContain('left: 40%');
		expect(thumb.classes()).toContain('bg-accent-400');
	});

	it('seeks to the clicked position', async () => {
		const w = mockRect(mountScrubber({ duration: 600 }));
		await w.trigger('click', { clientX: 50 });
		expect(w.emitted('seek')).toEqual([[300]]);
	});

	it('previews a time tooltip while dragging and seeks on release', async () => {
		const w = mockRect(mountScrubber({ duration: 600 }));
		dispatchPointer(w, 'pointerdown', { clientX: 0, pointerId: 1 });
		dispatchPointer(w, 'pointermove', { clientX: 50, pointerId: 1 });
		await Promise.resolve();

		const tooltip = w.find('[role="tooltip"]');
		expect(tooltip.exists()).toBe(true);
		expect(tooltip.text()).toBe('5:00 / 10:00');
		expect(w.emitted('seek')).toBeUndefined();

		dispatchPointer(w, 'pointerup', { clientX: 50, pointerId: 1 });
		await Promise.resolve();
		expect(w.emitted('seek')).toEqual([[300]]);
		expect(w.find('[role="tooltip"]').exists()).toBe(false);
	});

	it('does not seek or render a thumb when the duration is unknown', async () => {
		const w = mockRect(mountScrubber({ duration: 0 }));
		expect(w.find('[data-testid="player-progress-thumb"]').exists()).toBe(false);
		await w.trigger('click', { clientX: 50 });
		expect(w.emitted('seek')).toBeUndefined();
	});

	it('keeps chapter markers interactive via click', async () => {
		const w = mountScrubber({
			chapterMarkers: [{ left: 25, title: 'Main topic', startSeconds: 150 }]
		});
		await w.get('[data-testid="player-chapter-marker"]').trigger('click');
		expect(w.emitted('seek')).toEqual([[150]]);
	});
});
