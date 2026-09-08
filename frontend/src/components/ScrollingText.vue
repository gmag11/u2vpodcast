<script setup lang="ts">
	import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';

	const props = defineProps<{
		text: string;
		active: boolean;
	}>();

	const viewport = ref<HTMLElement | null>(null);
	const textElement = ref<HTMLElement | null>(null);
	const scrollDistance = ref(0);
	const reducedMotion = ref(false);
	const SCROLL_GAP_PX = 32;
	const SCROLL_SPEED_PX_PER_SECOND = 32;
	let resizeObserver: ResizeObserver | undefined;

	const scrollActive = computed(
		() => props.active && !reducedMotion.value && scrollDistance.value > 0
	);
	const scrollStyle = computed(() => ({
		'--scrolling-text-distance': `${scrollDistance.value}px`,
		'--scrolling-text-duration': `${scrollDistance.value / SCROLL_SPEED_PX_PER_SECOND}s`
	}));

	async function measure() {
		await nextTick();
		const viewportWidth = viewport.value?.clientWidth ?? 0;
		const textWidth = textElement.value?.scrollWidth ?? 0;
		scrollDistance.value =
			viewportWidth > 0 && textWidth > viewportWidth ? textWidth + SCROLL_GAP_PX : 0;
	}

	watch(
		() => props.text,
		() => void measure()
	);

	onMounted(() => {
		reducedMotion.value =
			typeof window.matchMedia === 'function' &&
			window.matchMedia('(prefers-reduced-motion: reduce)').matches;
		void measure();
		if (typeof ResizeObserver !== 'undefined') {
			resizeObserver = new ResizeObserver(() => void measure());
			if (viewport.value) resizeObserver.observe(viewport.value);
			if (textElement.value) resizeObserver.observe(textElement.value);
		}
	});

	onBeforeUnmount(() => resizeObserver?.disconnect());
</script>

<template>
	<div
		ref="viewport"
		class="scrolling-text-viewport overflow-hidden"
		data-testid="scrolling-text-viewport"
		:aria-label="text"
	>
		<span
			class="block min-w-full whitespace-nowrap"
			:class="
				scrollActive
					? 'scrolling-text-track scrolling-text-track--active inline-flex w-max'
					: 'truncate'
			"
			:style="scrollStyle"
			data-testid="scrolling-text-track"
		>
			<span ref="textElement" class="inline-block shrink-0" data-testid="scrolling-text-text">{{
				text
			}}</span>
			<span
				v-if="scrollDistance > 0"
				class="shrink-0"
				data-testid="scrolling-text-copy"
				aria-hidden="true"
			>
				{{ text }}
			</span>
		</span>
	</div>
</template>

<style scoped>
	.scrolling-text-viewport {
		container-type: inline-size;
	}

	.scrolling-text-track {
		gap: 32px;
	}

	.scrolling-text-track--active {
		animation: scrolling-text var(--scrolling-text-duration) linear infinite;
		will-change: transform;
	}

	@keyframes scrolling-text {
		from {
			transform: translateX(0);
		}
		to {
			transform: translateX(calc(-1 * var(--scrolling-text-distance)));
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.scrolling-text-track--active {
			animation: none;
		}
	}
</style>
