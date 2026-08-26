import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import type { Episode, EpisodeProgress } from '@/types';
import { loadQueue, saveQueue } from '@/lib/utils/queue.storage';
import { api } from '@/lib/api/client';

// Upper bound for the playback history so long sessions cannot grow it
// without limit (its only purpose is the previous control).
const PLAY_STACK_LIMIT = 50;

// How often the position is persisted while playing (playback-progress).
const SAVE_INTERVAL_MS = 10_000;
// Resume threshold: below 30s the episode starts from the beginning.
// Exported so the episode cards reuse the same resume gesture boundary.
export const RESUME_POSITION_S = 30;
// Positions within 95% of the duration count as finished (no resume).
const RESUME_DURATION_RATIO = 0.95;
// Keyboard seek step (playback-progress shortcuts).
const KEYBOARD_SEEK_STEP = 15;

// Playback-progress debug traces only in development builds; production
// carries no `[player]` console noise. Console: DevTools > Console (F12).
const DEBUG_PLAYER = import.meta.env.DEV;

function trace(...args: unknown[]) {
	if (DEBUG_PLAYER) {
		console.debug('[player]', ...args);
	}
}

// Parses the stored duration string (`H:MM:SS`, `M:SS` or plain seconds) into
// seconds. Used only as a fallback when the media element has no usable
// duration, so a completed episode never records a zero position.
function parseDurationSeconds(raw: string | null | undefined): number | null {
	if (raw == null) return null;
	const parts = raw.split(':').map((p) => Number(p));
	if (parts.length === 0 || parts.some((p) => !isFinite(p) || p < 0)) return null;
	return parts.reduce((acc, p) => acc * 60 + p, 0);
}

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

	// Playback progress keyed by episode id. The same episode exists as several
	// object copies (list item, playlist/queue item, restored queue), so the
	// authoritative value is tracked per id and every copy observes it,
	// independent of where the episode was played from (playback-progress).
	const progressByEpisode = ref<Record<number, EpisodeProgress>>({});

	let audio: HTMLAudioElement | null = null;

	// Playback-progress bookkeeping: wall-clock gate for the throttled saves,
	// the last (episode, position) actually sent, and the pending resume flag
	// (with its target episode id).
	let lastSaveAt = 0;
	let lastSavedPosition = -1;
	let lastSavedEpisodeId: number | null = null;
	let resumePending = false;
	let resumeEpisodeId: number | null = null;
	// Resume target + retry loop. Browsers with `preload=metadata` limit
	// `seekable` to the buffered prefix and silently clamp single seeks beyond
	// it; the retry re-issues the seek as the buffer (and thus `seekable`)
	// grows while playing, so it lands without forcing a full download.
	let resumeTarget: number | null = null;
	let resumeDeadline = 0;
	let resumeTimer: ReturnType<typeof setInterval> | null = null;

	function recordProgress(episode: Episode, progress: { position_seconds: number; listen: boolean; listened_at: string | null }) {
		episode.position_seconds = progress.position_seconds;
		episode.listen = progress.listen;
		episode.listened_at = progress.listened_at;
		progressByEpisode.value = {
			...progressByEpisode.value,
			[episode.id]: {
				id: episode.id,
				yt_id: episode.yt_id,
				...progress
			}
		};
	}

	// Resolves the authoritative progress for an episode id: the recorded value
	// when this session touched it, otherwise the episode's own fields.
	function effectiveProgress(episode: Episode): EpisodeProgress {
		return (
			progressByEpisode.value[episode.id] ?? {
				id: episode.id,
				yt_id: episode.yt_id,
				position_seconds: episode.position_seconds,
				listen: episode.listen,
				listened_at: episode.listened_at ?? null
			}
		);
	}

	// Returns the episode copy merged with its per-id progress, so consumers
	// (e.g. cards) always reflect the latest saved values without a refetch.
	function episodeWithProgress(episode: Episode): Episode {
		return { ...episode, ...effectiveProgress(episode) };
	}

	// Records the progress already carried by a freshly fetched episode list
	// (the episode endpoints include `position_seconds`/`listen`/`listened_at`),
	// so resume works without a per-play request. Live entries from this
	// session are kept.
	function seedProgress(episodes: Episode[]) {
		for (const episode of episodes) {
			if (episode.id == null || episode.yt_id == null) continue;
			if (progressByEpisode.value[episode.id] != null) continue;
			progressByEpisode.value = {
				...progressByEpisode.value,
				[episode.id]: {
					id: episode.id,
					yt_id: episode.yt_id,
					position_seconds: episode.position_seconds ?? 0,
					listen: episode.listen ?? false,
					listened_at: episode.listened_at ?? null
				}
			};
		}
	}

	function persistQueue() {
		saveQueue({
			upNext: upNext.value,
			playStack: playStack.value,
			currentEpisode: currentEpisode.value
		});
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
			currentEpisode.value = stored.currentEpisode;
		}
	}

	// Window-level progress plumbing: keyboard seeks (ArrowRight/ArrowLeft) and
	// a final position flush when the tab is hidden or closed. The store is a
	// singleton for the whole SPA session, so these are installed once.
	if (typeof window !== 'undefined') {
		window.addEventListener('keydown', onWindowKeydown);
		const flushOnUnload = () => persistProgress();
		window.addEventListener('pagehide', flushOnUnload);
		document.addEventListener('visibilitychange', () => {
			if (document.visibilityState === 'hidden') persistProgress();
		});
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
		tryResumeSeek();
		// Periodic saves only make sense while actually playing: a stopped or
		// paused element must not keep persisting positions every 10s.
		if (!audio || audio.paused || stopped.value) return;
		const now = Date.now();
		if (now - lastSaveAt >= SAVE_INTERVAL_MS) {
			lastSaveAt = now;
			persistProgress();
		}
	}

	function onLoadedMetadata() {
		if (audio) duration.value = audio.duration;
		// The duration became known: re-evaluate the pending resume.
		tryResumeSeek();
	}

	// Clears the pending resume state and stops the retry loop.
	function finishResume() {
		resumePending = false;
		resumeEpisodeId = null;
		resumeTarget = null;
		resumeDeadline = 0;
		if (resumeTimer) {
			clearInterval(resumeTimer);
			resumeTimer = null;
		}
	}

	// Starts the resume retry loop for `target`. Browsers with a metadata-only
	// preload limit `seekable` to the buffered prefix and clamp a single seek
	// beyond it; while playback continues the buffer (and `seekable`) grows, so
	// periodically re-issuing the seek eventually lands it — without forcing a
	// full download (playback-progress).
	function startResume(target: number) {
		resumePending = true;
		resumeEpisodeId = currentEpisode.value?.id ?? null;
		resumeTarget = target;
		resumeDeadline = Date.now() + 20_000;
		const current = currentEpisode.value;
		if (current) {
			// Keep the canonical progress in sync with what we are resuming to.
			recordProgress(current, { ...effectiveProgress(current), position_seconds: target });
		}
		if (resumeTimer) clearInterval(resumeTimer);
		resumeTimer = setInterval(tryResumeSeek, 500);
		tryResumeSeek();
	}

	// One attempt of the resume retry loop.
	function tryResumeSeek() {
		if (resumeTarget == null) return;
		const el = audio;
		const current = currentEpisode.value;
		if (!el || !current || current.id !== resumeEpisodeId) {
			finishResume();
			return;
		}
		if (Date.now() > resumeDeadline) {
			trace('resume: could not reach the target, continuing', {
				target: resumeTarget,
				at: el.currentTime
			});
			finishResume();
			return;
		}
		if (isFinite(el.duration) && el.duration > 0 && resumeTarget >= el.duration * RESUME_DURATION_RATIO) {
			// Near the end: treat as finished, start from zero.
			trace('resume: playing from zero (near end)');
			el.currentTime = 0;
			currentTime.value = 0;
			finishResume();
			return;
		}
		if (el.currentTime >= resumeTarget - 1) {
			trace('resume: at target', resumeTarget);
			finishResume();
			return;
		}
		trace('resume: seek attempt', { target: resumeTarget, at: el.currentTime });
		el.currentTime = resumeTarget;
		currentTime.value = resumeTarget;
	}

	// Starts playback, arming the resume retry loop when one is pending.
	async function seekResumeOnPlay(el: HTMLAudioElement): Promise<void> {
		const current = currentEpisode.value;
		const pending =
			resumePending && current != null && current.id === resumeEpisodeId;
		if (!pending) {
			await el.play().catch(() => {});
			return;
		}
		const effective = effectiveProgress(current!);
		if (effective.position_seconds > RESUME_POSITION_S) {
			startResume(effective.position_seconds);
		} else {
			trace('resume: playing from zero');
			finishResume();
		}
		await el.play().catch(() => {});
	}

	function onPlay() {
		playing.value = true;
		loading.value = false;
	}

	function onPause() {
		playing.value = false;
		persistProgress();
	}

	function onWaiting() {
		loading.value = true;
	}

	function onCanPlay() {
		loading.value = false;
	}

	function onEnded() {
		playing.value = false;
		markListened();
		advance();
	}

	// Fire-and-forget progress write. `listened` overrides the mark decision
	// (completion / long-press skip); otherwise the episode's current mark is
	// preserved so routine saves never accidentally clear it. Positions that
	// have not moved are skipped unless a mark change needs persisting.
	function persistProgress(listened?: boolean, position?: number) {
		const episode = currentEpisode.value;
		const el = audio;
		if (!episode || !el) return;
		// A stopped episode is already finalized by `stop()`; late events (the
		// asynchronously delivered `pause`, pagehide after stop) must not
		// overwrite the saved position with the reset zero playhead.
		if (stopped.value) return;
		// While a resume is pending for this episode its stored position is the
		// resume target and remains authoritative: premature saves (the playhead
		// before the seek lands) must not overwrite it with the buffered prefix.
		// Persisting the target itself keeps a pause/tab-close inside the retry
		// window from losing the final position.
		if (resumePending && resumeEpisodeId === episode.id) {
			position = position ?? resumeTarget ?? el.currentTime;
		}
		const pos = Math.floor(position ?? el.currentTime);
		const effective = effectiveProgress(episode);
		const mark = listened ?? effective.listen;
		// Never regress the position of a listened episode: a late flush (e.g.
		// after `skipNext(markCurrent)` finalizes it at its duration) must not
		// overwrite the completion position with the live playhead.
		if (mark && pos < episode.position_seconds) return;
		if (lastSavedEpisodeId === episode.id && pos === lastSavedPosition && !mark) return;
		lastSavedPosition = pos;
		lastSavedEpisodeId = episode.id;
		trace('save position', episode.yt_id, { position_seconds: pos, listened: mark });
		recordProgress(episode, {
			position_seconds: pos,
			listen: mark,
			listened_at: mark ? (effective.listened_at ?? new Date().toISOString()) : null
		});
		api
			.updateEpisodeProgress(episode.yt_id, {
				position_seconds: pos,
				listened: mark
			})
			.catch((err) => {
				console.error('Failed to save playback progress', err);
			});
	}

	// Shared listened-mark path: completion (`ended`) and the long-press next
	// gesture (`skipNext`) both record the episode as played at its duration.
	function markListened() {
		const episode = currentEpisode.value;
		if (!episode) return;
		const position = Math.floor(
			audio && isFinite(audio.duration) && audio.duration > 0
				? audio.duration
				: (parseDurationSeconds(episode.duration) ?? 0)
		);
		const listenedAt = new Date().toISOString();
		episode.listen = true;
		episode.listened_at = listenedAt;
		episode.position_seconds = position;
		currentEpisode.value = { ...episode };
		lastSavedPosition = position;
		lastSavedEpisodeId = episode.id;
		recordProgress(episode, {
			position_seconds: position,
			listen: true,
			listened_at: listenedAt
		});
		api.updateEpisodeProgress(episode.yt_id, {
			position_seconds: position,
			listened: true
		}).catch((err) => {
			console.error('Failed to save listened mark', err);
		});
	}

	function mediaUrl(episode: Episode) {
		return `/media/${episode.channel_slug}/${episode.yt_id}.mp3`;
	}

	// Resolves once the element's metadata is available. A 5s safety net keeps
	// playback from blocking forever on media that never exposes metadata (the
	// timer is cleared as soon as `loadedmetadata` fires).
	function waitForMetadata(el: HTMLAudioElement): Promise<void> {
		return new Promise((resolve) => {
			const timer = setTimeout(resolve, 5000);
			const onMeta = () => {
				clearTimeout(timer);
				el.removeEventListener('loadedmetadata', onMeta);
				resolve();
			};
			el.addEventListener('loadedmetadata', onMeta);
		});
	}

	async function loadEpisode(episode: Episode) {
		const el = ensureAudio();
		if (!el) return;
		const isSame = currentEpisode.value != null && currentEpisode.value.id === episode.id;
		if (!isSame && !stopped.value) {
			// Flush the departing episode's playhead before switching sources so
			// nothing between throttle saves is lost (playback-progress).
			persistProgress();
		}
		currentEpisode.value = episode;
		stopped.value = false;
		const reloading = !isSame || el.src === '';
		if (reloading) {
			// Wait for the metadata before playback starts so the resume has the
			// duration at hand (playback-progress).
			const meta = waitForMetadata(el);
			el.src = mediaUrl(episode);
			el.load();
			// A freshly loaded source starts at elapsed 0; clear the store
			// playhead so consumers never see the previous episode's position.
			currentTime.value = 0;
			await meta;
		}
		await seekResumeOnPlay(el);
	}

	// Arms the resume decision for an incoming episode before its source loads:
	// the per-id progress map when the session already knows the episode,
	// otherwise a one-shot server lookup. `seekResumeOnPlay` (reached inside
	// `loadEpisode`) consumes the pending flag. Shared by `play()`, the
	// restored-queue restart (`togglePlay`) and queue navigation (`advance()`,
	// `skipNext()`, `playPrevious()`) so that every way of landing on an
	// episode respects its saved start time (playback-progress).
	async function armResume(episode: Episode, label: string) {
		const known = progressByEpisode.value[episode.id] != null;
		if (!known) {
			const result = await api.getEpisodeProgress(episode.yt_id).catch(() => null);
			const fetched = result?.ok ? result.data : result;
			trace(`${label}: server progress`, episode.yt_id, fetched);
			if (result?.ok && result.data) {
				recordProgress(episode, {
					position_seconds: result.data.position_seconds,
					listen: result.data.listen,
					listened_at: result.data.listened_at ?? null
				});
			}
		}
		const stored = effectiveProgress(episode);
		resumePending = stored.position_seconds > RESUME_POSITION_S;
		resumeEpisodeId = resumePending ? episode.id : null;
		trace(`${label}: resume decision`, {
			yt_id: episode.yt_id,
			source: known ? 'list' : 'server',
			effective_position_seconds: stored.position_seconds,
			resumePending
		});
	}

	async function play(episode: Episode, list?: Episode[], opts?: { fromStart?: boolean }) {
		const fromStart = opts?.fromStart ?? false;
		if (list) {
			const index = list.findIndex((e) => e.id === episode.id);
			upNext.value = index < 0 ? [] : list.slice(index + 1);
		}
		// Without a context list the existing queue is kept untouched, so a
		// single-episode play (e.g. replaying something already queued) does
		// not wipe the up-next flow.
		if (fromStart) {
			// "start over": no resume. finishResume also stops the retry loop if
			// a previous resume was still pending; the live playhead is reset so
			// replaying the already-loaded current episode really restarts.
			finishResume();
			if (audio) {
				audio.currentTime = 0;
				currentTime.value = 0;
			}
		} else {
			// The episode list already seeded the per-id progress; the resume
			// decision is shared with every other navigation path (also covers
			// episodes unknown to this session via a one-shot lookup, e.g. a
			// restored-queue entry not present in any fetched list).
			await armResume(episode, 'play');
		}
		await loadEpisode(episode);
		if (fromStart) {
			currentEpisode.value = { ...episode, position_seconds: 0 };
			persistProgress(undefined, 0);
		}
		persistQueue();
	}

	async function advance() {
		const finished = currentEpisode.value;
		const next = upNext.value.shift();
		if (next) {
			if (finished) pushToPlayStack(finished);
			// The next queued episode resumes from its saved position when it
			// has one, just like a fresh `play()` (playback-progress).
			await armResume(next, 'advance');
			await loadEpisode(next);
			persistQueue();
		} else {
			stop();
			upNext.value = [];
			persistQueue();
		}
	}

	async function skipNext(markCurrent: boolean = false) {
		const finished = currentEpisode.value;
		if (markCurrent && finished) {
			// Long-press skip records the current episode exactly like
			// completion (listen=true, listened_at set, position=duration).
			markListened();
		}
		const next = upNext.value.shift();
		if (next) {
			if (finished) pushToPlayStack(finished);
			await armResume(next, 'skipNext');
			await loadEpisode(next);
			persistQueue();
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
			// Returning to a previously played episode resumes it, matching the
			// design promise for the dual previous control (playback-progress).
			await armResume(previous, 'playPrevious');
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
			const wasStopped = stopped.value;
			stopped.value = false;
			// After a reload/restore the shared element may have been created
			// with an empty source: (re)load the current episode before playing.
			const reloading = !el.src || el.src === '';
			if (reloading) {
				// Wait for metadata before playback starts so the resume seek
				// lands deterministically (see `loadEpisode`).
				const meta = waitForMetadata(el);
				el.src = mediaUrl(currentEpisode.value);
				el.load();
				await meta;
			}
			if (wasStopped) {
				// Replaying a stopped episode follows the same resume policy as
				// a fresh `play()` (playback-progress).
				const episode = currentEpisode.value;
				if (episode) {
					await armResume(episode, 'togglePlay');
				}
			}
			// Play and, when a resume is pending, retry the seek as the buffer
			// grows (browsers otherwise clamp a seek beyond `seekable`).
			await seekResumeOnPlay(el);
		} else {
			el.pause();
		}
	}

	async function pause() {
		if (audio) audio.pause();
	}

	function stop() {
		// Flush the final position before the element is reset to zero.
		persistProgress();
		finishResume();
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

	// Keyboard shortcut: seek ±15s clamped to the episode bounds. Persisted by
	// the existing throttled/event-driven saves, exactly like scrubber seeks.
	function seekRelative(delta: number) {
		if (!audio || !currentEpisode.value) return;
		const max = isFinite(audio.duration) && audio.duration > 0 ? audio.duration : 0;
		const next = Math.min(Math.max(audio.currentTime + delta, 0), max);
		audio.currentTime = next;
		currentTime.value = next;
	}

	function onWindowKeydown(event: KeyboardEvent) {
		if (event.key !== 'ArrowRight' && event.key !== 'ArrowLeft') return;
		if (!document.hasFocus()) return;
		if (!currentEpisode.value || !audio) return;
		const target = event.target as HTMLElement | null;
		if (!target) return;
		const tag = target.tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || target.isContentEditable) {
			return;
		}
		if (typeof target.closest === 'function' && target.closest('[role=slider]')) {
			return;
		}
		event.preventDefault();
		seekRelative(event.key === 'ArrowRight' ? KEYBOARD_SEEK_STEP : -KEYBOARD_SEEK_STEP);
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
		seekRelative,
		episodeWithProgress,
		seedProgress,
		setVolume,
		toggleMute,
		setSpeed,
		isCurrent
	};
});