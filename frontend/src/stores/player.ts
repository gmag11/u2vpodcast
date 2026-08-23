import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import type { Episode } from '@/types';
import { loadQueue, saveQueue } from '@/lib/utils/queue.storage';

// Upper bound for the playback history so long sessions cannot grow it
// without limit (its only purpose is the previous control).
const PLAY_STACK_LIMIT = 50;

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
	const playStack = ref<Episode[]>([]);

	let audio: HTMLAudioElement | null = null;

	function persistQueue() {
		saveQueue({ upNext: upNext.value, playStack: playStack.value });
	}

	function pushToPlayStack(episode: Episode) {
		playStack.value.push(episode);
		if (playStack.value.length > PLAY_STACK_LIMIT) {
			playStack.value.splice(0, playStack.value.length - PLAY_STACK_LIMIT);
		}
	}

	// Rehydrate the persisted queue once at store creation. A malformed payload
	// is discarded by queue.storage and leaves the queue empty.
	{
		const stored = loadQueue();
		if (stored) {
			upNext.value = stored.upNext;
			playStack.value = stored.playStack;
		}
	}

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
			persistQueue();
		}
		// Without a context list the existing queue is kept untouched, so a
		// single-episode play (e.g. replaying something already queued) does
		// not wipe the up-next flow.
		await loadEpisode(episode);
	}

	async function advance() {
		const finished = currentEpisode.value;
		const next = upNext.value.shift();
		if (next) {
			if (finished) pushToPlayStack(finished);
			persistQueue();
			await loadEpisode(next);
		} else {
			stop();
			upNext.value = [];
			persistQueue();
		}
	}

	async function skipNext(markCurrent: boolean = false) {
		const finished = currentEpisode.value;
		if (markCurrent && finished) {
			// Local half of the "skip and mark listened" gesture. The server
			// persistence of the listened mark lands with step 3; the card sees
			// the updated state immediately.
			finished.listen = true;
			currentEpisode.value = { ...currentEpisode.value! };
		}
		const next = upNext.value.shift();
		if (next) {
			if (finished) pushToPlayStack(finished);
			persistQueue();
			await loadEpisode(next);
		} else {
			upNext.value = [];
			persistQueue();
		}
	}

	async function playPrevious() {
		// Dual behavior: past 3 seconds the previous control restarts the
		// current episode; within 3 seconds it navigates back in history.
		if (currentTime.value > 3) {
			if (audio) audio.currentTime = 0;
			currentTime.value = 0;
			return;
		}
		const previous = playStack.value.pop();
		if (previous) {
			persistQueue();
			await loadEpisode(previous);
		}
	}

	function removeFromQueue(episodeId: number) {
		upNext.value = upNext.value.filter((e) => e.id !== episodeId);
		persistQueue();
	}

	function clearQueue() {
		upNext.value = [];
		persistQueue();
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
		playStack,
		progress,
		currentLabel,
		durationLabel,
		play,
		advance,
		skipNext,
		playPrevious,
		removeFromQueue,
		clearQueue,
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