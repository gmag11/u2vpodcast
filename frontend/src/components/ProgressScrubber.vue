<script setup lang="ts">
	import { computed, ref } from 'vue';

	interface SponsorBlockMarker {
		left: number;
		width: number;
		category: string;
	}

	interface ChapterMarker {
		left: number;
		title: string;
		startSeconds: number;
	}

	const props = withDefaults(
		defineProps<{
			progress: number;
			duration: number;
			sponsorBlockMarkers?: SponsorBlockMarker[];
			chapterMarkers?: ChapterMarker[];
			thin?: boolean;
			dataTestId?: string;
			ariaLabel?: string;
		}>(),
		{
			sponsorBlockMarkers: () => [],
			chapterMarkers: () => [],
			thin: false,
			dataTestId: undefined,
			ariaLabel: undefined
		}
	);

	const emit = defineEmits<{ seek: [seconds: number] }>();

	const trackEl = ref<HTMLElement | null>(null);
	const dragging = ref(false);
	const dragRatio = ref<number | null>(null);
	let pointerId = -1;
	let startX = 0;
	let moved = false;
	let suppressClick = false;

	const canSeek = computed(() => Number.isFinite(props.duration) && props.duration > 0);

	const thumbLeft = computed(() => {
		if (dragging.value && dragRatio.value != null) return dragRatio.value * 100;
		return Math.min(Math.max(props.progress, 0), 100);
	});

	const tooltipAnchor = computed(() => {
		if (thumbLeft.value < 15) return 'left-1/2';
		if (thumbLeft.value > 85) return 'right-1/2';
		return 'left-1/2 -translate-x-1/2';
	});

	// Matches the player's elapsed label format (M:SS / H:MM:SS, minutes unpadded).
	function formatElapsed(value: number) {
		const hours = Math.floor(value / 3600);
		const minutes = Math.floor((value % 3600) / 60);
		const seconds = Math.floor(value % 60);
		const minuteSeconds = `${minutes}:${String(seconds).padStart(2, '0')}`;
		return hours > 0
			? `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
			: minuteSeconds;
	}

	// Matches the player's total label format (zero-padded HH:MM:SS).
	function formatTotal(value: number) {
		const hours = Math.floor(value / 3600);
		const minutes = Math.floor((value % 3600) / 60);
		const seconds = Math.floor(value % 60);
		return [hours, minutes, seconds]
			.filter((v, i) => v > 0 || i > 0)
			.map((v) => String(v).padStart(2, '0'))
			.join(':');
	}

	const tooltipLabel = computed(() => {
		const ratio = dragging.value && dragRatio.value != null ? dragRatio.value : 0;
		return `${formatElapsed(ratio * props.duration)} / ${formatTotal(props.duration)}`;
	});

	function ratioFromClientX(clientX: number) {
		const el = trackEl.value;
		if (!el) return 0;
		const rect = el.getBoundingClientRect();
		if (rect.width <= 0) return 0;
		return Math.min(Math.max((clientX - rect.left) / rect.width, 0), 1);
	}

	// Chapter markers opt out of the scrubber's pointer handling so their own
	// activation (seek on click) keeps working.
	function isChapterMarker(target: EventTarget | null) {
		return target instanceof Element && target.closest('[data-chapter-marker]') != null;
	}

	function onPointerDown(event: PointerEvent) {
		if (!canSeek.value || isChapterMarker(event.target)) return;
		pointerId = event.pointerId;
		startX = event.clientX;
		moved = false;
		suppressClick = false;
		dragging.value = true;
		dragRatio.value = ratioFromClientX(event.clientX);
		if (trackEl.value?.setPointerCapture) {
			try {
				trackEl.value.setPointerCapture(event.pointerId);
			} catch {
				// jsdom and some browsers refuse capture for synthetic pointers.
			}
		}
	}

	function onPointerMove(event: PointerEvent) {
		if (!dragging.value || event.pointerId !== pointerId) return;
		dragRatio.value = ratioFromClientX(event.clientX);
		if (Math.abs(event.clientX - startX) > 3) moved = true;
	}

	function onPointerUp(event: PointerEvent) {
		if (!dragging.value || event.pointerId !== pointerId) return;
		const ratio = dragRatio.value ?? ratioFromClientX(event.clientX);
		const wasDrag = moved;
		dragging.value = false;
		dragRatio.value = null;
		if (trackEl.value?.hasPointerCapture && trackEl.value.hasPointerCapture(pointerId)) {
			trackEl.value.releasePointerCapture(pointerId);
		}
		if (wasDrag) {
			// A real drag already seeks here; swallow the trailing click so the
			// position isn't re-read (and re-seeked) from a slightly moved cursor.
			suppressClick = true;
			emit('seek', ratio * props.duration);
		}
	}

	function onPointerCancel() {
		dragging.value = false;
		dragRatio.value = null;
	}

	function onClick(event: MouseEvent) {
		if (suppressClick) {
			suppressClick = false;
			return;
		}
		if (!canSeek.value || isChapterMarker(event.target)) return;
		emit('seek', ratioFromClientX(event.clientX) * props.duration);
	}
</script>

<template>
	<div
		ref="trackEl"
		class="relative h-5 w-full cursor-pointer touch-none py-2"
		role="slider"
		:aria-label="ariaLabel"
		:aria-valuenow="Math.round(thumbLeft)"
		aria-valuemin="0"
		aria-valuemax="100"
		:data-testid="dataTestId"
		@pointerdown="onPointerDown"
		@pointermove="onPointerMove"
		@pointerup="onPointerUp"
		@pointercancel="onPointerCancel"
		@click="onClick"
	>
		<div class="relative w-full rounded-full bg-surface-input" :class="thin ? 'h-1' : 'h-1.5'">
			<div
				class="absolute inset-y-0 left-0 rounded-full bg-accent-400 transition-[width] duration-150"
				:style="{ width: thumbLeft + '%' }"
			></div>
			<div
				v-for="(marker, index) in sponsorBlockMarkers"
				:key="`s-${index}`"
				class="absolute inset-y-0 z-10"
				:class="marker.category === 'sponsor' ? 'bg-sponsorblock' : 'bg-sponsorblock-other'"
				:data-category="marker.category"
				data-testid="player-sponsorblock-segment"
				:style="{ left: `${marker.left}%`, width: `${marker.width}%` }"
			></div>
			<button
				v-for="(marker, index) in chapterMarkers"
				:key="index"
				type="button"
				class="group absolute -inset-y-2 z-20 w-3 -translate-x-1/2"
				data-testid="player-chapter-marker"
				data-chapter-marker
				:data-start-seconds="marker.startSeconds"
				:aria-label="marker.title"
				:aria-describedby="`scrubber-chapter-tooltip-${index}`"
				:style="{ left: `${marker.left}%` }"
				@click.stop="emit('seek', marker.startSeconds)"
			>
				<span
					class="absolute inset-y-2 left-1/2 w-0.5 -translate-x-1/2 bg-chapter-marker"
					aria-hidden="true"
				></span>
				<span
					:id="`scrubber-chapter-tooltip-${index}`"
					role="tooltip"
					class="pointer-events-none absolute bottom-full z-30 mb-1 w-max max-w-64 rounded-md bg-surface-high px-2 py-1 text-left text-xs font-medium text-text opacity-0 shadow-card transition-opacity group-hover:opacity-100 group-focus-visible:opacity-100"
					:class="
						marker.left < 15
							? 'left-1/2'
							: marker.left > 85
								? 'right-1/2'
								: 'left-1/2 -translate-x-1/2'
					"
				>
					{{ marker.title }}
				</span>
			</button>
			<div
				v-if="canSeek"
				class="absolute top-1/2 z-30 h-3 w-3 -translate-x-1/2 -translate-y-1/2 rounded-full bg-accent-400 shadow-md"
				:style="{ left: thumbLeft + '%' }"
				data-testid="player-progress-thumb"
			>
				<span
					v-if="dragging"
					role="tooltip"
					class="pointer-events-none absolute bottom-full mb-2 w-max rounded-md bg-surface-high px-2 py-1 text-xs font-medium tabular-nums text-text shadow-card"
					:class="tooltipAnchor"
				>
					{{ tooltipLabel }}
				</span>
			</div>
		</div>
	</div>
</template>
