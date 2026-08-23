import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import type { Episode } from '@/types';

export const usePlayerStore = defineStore('player', () => {
	const currentEpisode = ref<Episode | null>(null);
	const playing = ref(false);
	const currentTime = ref(0);
	const duration = ref(0);
	const volume = ref(1);
	const muted = ref(false);
	const speed = ref(1);
	const loading = ref(false);
	const stopped = ref(true);
	const upNext = ref<Episode[]>([]);

	let audio: HTMLAudioElement | null = null;

	function ensureAudio(): HTMLAudioElement | null {
		if (typeof window === 'undefined') return null;
		if (!audio) {
			audio = new Audio();
			audio.preload = 'metadata';
			audio.volume = volume.value;
			audio.addEventListener('timeupdate', onTimeUpdate);
			audio.addEventListener('loadedmetadata', onLoadedMetadata);
			audio.addEventListener('play', onPlay);
			audio.addEventListener('pause', onPause);
			audio.addEventListener('waiting', onWaiting);
			audio.addEventListener('canplay', onCanPlay);
			audio.addEventListener('ended', onEnded);
		}
		return audio;
	}

	function onTimeUpdate() {
		if (audio) currentTime.value = audio.currentTime;
	}

	function onLoadedMetadata() {
		if (audio) duration.value = audio.duration;
	}

	function onPlay() {
		playing.value = true;
		loading.value = false;
	}

	function onPause() {
		playing.value = false;
	}

	function onWaiting() {
		loading.value = true;
	}

	function onCanPlay() {
		loading.value = false;
	}

	function onEnded() {
		playing.value = false;
		advance();
	}

	function mediaUrl(episode: Episode) {
		return `/media/${episode.channel_slug}/${episode.yt_id}.mp3`;
	}

	async function loadEpisode(episode: Episode) {
		const el = ensureAudio();
		if (!el) return;
		const isSame = currentEpisode.value != null && currentEpisode.value.id === episode.id;
		currentEpisode.value = episode;
		stopped.value = false;
		if (!isSame || el.src === '') {
			el.src = mediaUrl(episode);
			el.load();
		}
		await el.play();
	}

	async function play(episode: Episode, list?: Episode[]) {
		if (list) {
			const index = list.findIndex((e) => e.id === episode.id);
			upNext.value = index < 0 ? [] : list.slice(index + 1);
		} else {
			upNext.value = [];
		}
		await loadEpisode(episode);
	}

	async function advance() {
		const next = upNext.value.shift();
		if (next) {
			await loadEpisode(next);
		} else {
			stop();
		}
	}

	async function togglePlay() {
		const el = ensureAudio();
		if (!el || !currentEpisode.value) return;
		if (el.paused) {
			stopped.value = false;
			await el.play();
		} else {
			el.pause();
		}
	}

	async function pause() {
		if (audio) audio.pause();
	}

	function stop() {
		playing.value = false;
		stopped.value = true;
		currentTime.value = 0;
		if (audio) {
			audio.pause();
			audio.currentTime = 0;
		}
	}

	function seek(seconds: number) {
		if (audio) audio.currentTime = seconds;
	}

	function setVolume(value: number) {
		volume.value = value;
		if (audio) audio.volume = value;
	}

	function toggleMute() {
		muted.value = !muted.value;
		if (audio) audio.muted = muted.value;
	}

	function setSpeed(value: number) {
		speed.value = value;
		if (audio) audio.playbackRate = value;
	}

	const progress = computed(() =>
		duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0
	);

	const currentLabel = computed(() => {
		if (duration.value > 0) {
			const hours = Math.floor(currentTime.value / 3600);
			const minutes = Math.floor((currentTime.value % 3600) / 60);
			const seconds = Math.floor(currentTime.value % 60);
			return [hours, minutes, seconds]
				.filter((v, i) => v > 0 || i > 0)
				.map((v) => String(v).padStart(2, '0'))
				.join(':');
		}
		return '0:00';
	});

	const durationLabel = computed(() => {
		if (duration.value > 0) {
			const hours = Math.floor(duration.value / 3600);
			const minutes = Math.floor((duration.value % 3600) / 60);
			const seconds = Math.floor(duration.value % 60);
			return [hours, minutes, seconds]
				.filter((v, i) => v > 0 || i > 0)
				.map((v) => String(v).padStart(2, '0'))
				.join(':');
		}
		if (currentEpisode.value) {
			const raw = Number(currentEpisode.value.duration);
			if (!isNaN(raw) && raw > 0) {
				const hours = Math.floor(raw / 3600);
				const minutes = Math.floor((raw % 3600) / 60);
				const seconds = Math.floor(raw % 60);
				return [hours, minutes, seconds]
					.filter((v, i) => v > 0 || i > 0)
					.map((v) => String(v).padStart(2, '0'))
					.join(':');
			}
		}
		return '';
	});

	function isCurrent(episode: Episode) {
		return currentEpisode.value != null && currentEpisode.value.id === episode.id;
	}

	return {
		currentEpisode,
		playing,
		currentTime,
		duration,
		volume,
		muted,
		speed,
		loading,
		stopped,
		upNext,
		progress,
		currentLabel,
		durationLabel,
		play,
		advance,
		togglePlay,
		pause,
		stop,
		seek,
		setVolume,
		toggleMute,
		setSpeed,
		isCurrent
	};
});
