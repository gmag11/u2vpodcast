<script lang="ts">
	export let min = 0;
	export let max = 1;
	export let value: number;
	export let step = 0.01;
</script>

<div class="relative flex items-center flex-1">
	<div class="h-1 w-full rounded-sm bg-red-300 absolute"></div>
	<div
		class="h-1 rounded-m bg-gradient-to-r from-red-300 to-red-500 absolute"
		style={`width: ${(value * 100) / max}%;`}
	></div>
	<input
		bind:value
		{min}
		{max}
		{step}
		type="range"
		class="w-full absolute h-1 appearance-none bg-transparent outline-none"
	/>
</div>

<style>
	input[type='range']::-webkit-slider-thumb {
		-webkit-appearance: none;
		width: 4px;
		height: 4px;
		cursor: pointer;
	}

	input[type='range']::-webkit-slider-thumb:hover {
		background-color: #dc2626;
	}
</style>
