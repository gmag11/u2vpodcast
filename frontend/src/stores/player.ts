import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import type { Episode, EpisodeChapter, EpisodeProgress, SponsorBlockSegment } from '@/types';
import { loadQueue, saveQueue, type RepeatMode } from '@/lib/utils/queue.storage';
import { api } from '@/lib/api/client';
import { usePlaylistStore } from '@/stores/playlists';

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
// Keyboard and default system-media seek step (playback-progress shortcuts).
const KEYBOARD_SEEK_STEP = 15;
// Per-channel playback speed (per-channel-playback-speed): the stepper and
// the update API both work in half-tenth (0.05) steps within this range.
export const SPEED_MIN = 0.5;
export const SPEED_MAX = 3.0;
export const SPEED_STEP = 0.05;

const MEDIA_SESSION_ACTIONS: MediaSessionAction[] = [
	'play',
	'pause',
	'nexttrack',
	'previoustrack',
	'seekforward',
	'seekbackward',
	'seekto'
];

// Playback-progress debug traces only in development builds; production
// carries no `[player]` console noise. Console: DevTools > Console (F12).
const DEBUG_PLAYER = import.meta.env.DEV;

function trace(...args: unknown[]) {
	if (DEBUG_PLAYER) {
		console.debug('[player]', ...args);
	}
}

// Injectable source of randomness for the shuffle (playback-modes). Tests
// replace it with a seeded PRNG so shuffled orders are deterministic;
// production uses Math.random.
let randomSource: () => number = Math.random;

/**
 * Replaces the store's shuffle randomness source (test hook). Pass a seeded
 * PRNG to make `toggleShuffle()` / repeat-all rebuilds deterministic.
 */
export function setRandomSource(source: () => number): void {
	randomSource = source;
}

// Fisher–Yates shuffle over a copy; never mutates the input array. Returns a
// fresh, uniformly random permutation carrying the same episode references.
function shuffledCopy(episodes: Episode[]): Episode[] {
	const copy = [...episodes];
	for (let i = copy.length - 1; i > 0; i--) {
		const j = Math.floor(randomSource() * (i + 1));
		[copy[i], copy[j]] = [copy[j], copy[i]];
	}
	return copy;
}

// Parses the stored duration string (`H:MM:SS`, `M:SS` or plain seconds) into
// seconds. Used as a fallback when the media element has no usable duration
// and by the episode cards to size their read-only progress bar.
export function parseDurationSeconds(raw: string | null | undefined): number | null {
	if (raw == null) return null;
	const parts = raw.split(':').map((p) => Number(p));
	if (parts.length === 0 || parts.some((p) => !isFinite(p) || p < 0)) return null;
	return parts.reduce((acc, p) => acc * 60 + p, 0);
}

function activeSponsorBlockSegments(episode: Episode | null | undefined) {
	return episode?.sponsorblock_enabled === true ? episode.sponsorblock_segments : undefined;
}

export function sponsorBlockSkipTarget(
	seconds: number,
	segments: SponsorBlockSegment[] | null | undefined
): number {
	const rejected = (segments ?? [])
		.filter(
			({ start, end, rejected }) =>
				rejected && Number.isFinite(start) && Number.isFinite(end) && end > start
		)
		.sort((left, right) => left.start - right.start || left.end - right.end);
	const merged: Array<{ start: number; end: number }> = [];
	for (const segment of rejected) {
		const previous = merged.at(-1);
		if (previous && segment.start <= previous.end)
			previous.end = Math.max(previous.end, segment.end);
		else merged.push({ start: segment.start, end: segment.end });
	}
	const interval = merged.find(({ start, end }) => seconds >= start && seconds < end);
	return interval?.end ?? seconds;
}

// Mobile expanded view (expand-mobile-player-view): the three simplified
// states its combined shuffle/repeat control cycles through. Values outside
// this set (repeat-one, or shuffle combined with a repeat mode) have no exact
// representation; callers use `closestMobilePlaybackMode` to pick the nearest
// one without mutating the underlying shuffle/repeat state.
export type MobilePlaybackMode = 'normal' | 'repeat' | 'shuffle';

const MOBILE_PLAYBACK_MODE_CYCLE: MobilePlaybackMode[] = ['normal', 'repeat', 'shuffle'];

/**
 * Maps the store's independent `shuffle`/`repeat` state onto the closest of
 * the three mobile expanded-view states, without changing that state. Exact
 * matches: shuffle off + repeat none -> 'normal'; shuffle off + repeat all ->
 * 'repeat'; shuffle on + repeat none -> 'shuffle'. Anything else (repeat-one,
 * or shuffle combined with any repeat mode) falls back to 'shuffle' when
 * shuffle is on, otherwise 'repeat' (it is still repeating in some form).
 */
export function closestMobilePlaybackMode(
	shuffle: boolean,
	repeat: RepeatMode
): MobilePlaybackMode {
	if (!shuffle && repeat === 'none') return 'normal';
	if (!shuffle && repeat === 'all') return 'repeat';
	if (shuffle && repeat === 'none') return 'shuffle';
	return shuffle ? 'shuffle' : 'repeat';
}

/** Returns the next state in the normal -> repeat -> shuffle -> normal cycle. */
export function nextMobilePlaybackMode(current: MobilePlaybackMode): MobilePlaybackMode {
	const index = MOBILE_PLAYBACK_MODE_CYCLE.indexOf(current);
	return MOBILE_PLAYBACK_MODE_CYCLE[(index + 1) % MOBILE_PLAYBACK_MODE_CYCLE.length];
}

export function sponsorBlockTimelineMarkers(
	duration: number,
	segments: SponsorBlockSegment[] | null | undefined
): Array<{ left: number; width: number; category: string }> {
	if (!Number.isFinite(duration) || duration <= 0) return [];
	return (segments ?? []).flatMap(({ start, end, category }) => {
		const clampedStart = Math.min(Math.max(start, 0), duration);
		const clampedEnd = Math.min(Math.max(end, 0), duration);
		if (!Number.isFinite(start) || !Number.isFinite(end) || clampedEnd <= clampedStart) return [];
		return [
			{
				left: (clampedStart / duration) * 100,
				width: ((clampedEnd - clampedStart) / duration) * 100,
				category
			}
		];
	});
}

export function chapterTimelineMarkers(
	duration: number,
	chapters: EpisodeChapter[] | null | undefined
): Array<{ left: number; title: string; startSeconds: number }> {
	if (!Number.isFinite(duration) || duration <= 0) return [];
	return (chapters ?? []).flatMap(({ start, title }) => {
		if (!Number.isFinite(start) || start < 0 || start > duration) return [];
		return [{ left: (start / duration) * 100, title, startSeconds: start }];
	});
}

export function currentChapterIndex(
	currentTime: number,
	chapters: EpisodeChapter[] | null | undefined
): number {
	return (chapters ?? []).findIndex(({ start, end }) => currentTime >= start && currentTime < end);
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
	// Authored order of the last seeded queue (playback-modes). This is the
	// "source" for shuffle and repeat-all: shuffle reorders the consumption
	// order (`upNext`) but never this array, disabling shuffle restores it, and
	// repeat-all rebuilds from it when the queue drains. Re-seeded by play(),
	// cleared by clearQueue(); user removals leave it too. Not shown in the
	// queue panel — that renders `upNext`.
	const seedOrder = ref<Episode[]>([]);
	// Playback modes (playback-modes). `repeat` cycles none → all → one.
	const shuffle = ref(false);
	const repeat = ref<RepeatMode>('none');
	// Origin of the current queue seed: 'playlist' when play() was called from
	// the playlist view, 'list' otherwise. Consumed on completion / long-press
	// skip to decide whether the finished episode also leaves the playlist
	// (playlist-capability). Re-seeded on every play(); not persisted.
	const queueSource = ref<'playlist' | 'list'>('list');

	// Playback progress keyed by episode id. The same episode exists as several
	// object copies (list item, playlist/queue item, restored queue), so the
	// authoritative value is tracked per id and every copy observes it,
	// independent of where the episode was played from (playback-progress).
	const progressByEpisode = ref<Record<number, EpisodeProgress>>({});
	// Saved playback speed per channel slug (per-channel-playback-speed),
	// seeded from episode payloads and persisted with the queue so a reloaded
	// session starts episodes at the right rate.
	const channelSpeedBySlug = ref<Record<string, number>>({});

	let audio: HTMLAudioElement | null = null;
	let mediaSessionRegistered = false;
	let mediaSessionGeneration = 0;

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
	// Id of the episode just finalized at its duration (completion / long-press
	// skip). While set, a departing-episode flush must not regress that
	// completion position; it is cleared as soon as playback restarts on any
	// episode, so live re-listens save normally again.
	let finalizedEpisodeId: number | null = null;

	function recordProgress(
		episode: Episode,
		progress: { position_seconds: number; listen: boolean; listened_at: string | null }
	) {
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

	// Public progress recorder: lets cards immediately reflect a server-confirmed
	// progress change (e.g. unmarking a listened episode) through the shared
	// per-id map instead of waiting for a refetch.
	function applyProgress(
		episode: Episode,
		progress: { position_seconds: number; listen: boolean; listened_at: string | null }
	) {
		recordProgress(episode, progress);
	}

	// Records the channel's saved speed from any episode that carries one, so
	// the per-channel map stays fresh without an extra request
	// (per-channel-playback-speed). Only fills unknown entries: an existing
	// entry reflects this session's (potentially newer) knowledge — e.g. a
	// speed the user just saved — and must not be clobbered by a stale value
	// carried on an episode fetched before that change.
	function seedChannelSpeed(episode: Episode) {
		if (!episode.channel_slug || episode.playback_speed == null) return;
		if (channelSpeedBySlug.value[episode.channel_slug] != null) return;
		channelSpeedBySlug.value = {
			...channelSpeedBySlug.value,
			[episode.channel_slug]: episode.playback_speed
		};
	}

	// Applies the saved speed of the episode's channel to the shared element.
	// `audio.playbackRate` is a persistent property that survives `src`
	// changes, so it MUST be rewritten on every episode load: this runs on
	// every source-load path (play, end-of-episode auto-advance, manual skip,
	// restored-queue restart) so a cross-channel switch loads and applies the
	// NEW channel's value and never inherits the previous channel's rate.
	// Resolution order: this session's known map entry first (it reflects the
	// latest user action, e.g. after a reload the persisted map beats the
	// restored episode's payload), then the episode's own payload value, then
	// the 1.0 default.
	function applyChannelSpeed(episode: Episode) {
		seedChannelSpeed(episode);
		const known = channelSpeedBySlug.value[episode.channel_slug];
		const next = known ?? episode.playback_speed ?? 1.0;
		speed.value = next;
		if (audio) {
			// Browsers reset playbackRate to defaultPlaybackRate when load()
			// runs (fire-and-forget retargets), so both properties must carry
			// the channel's saved value: playbackRate is the live rate,
			// defaultPlaybackRate is what a subsequent load() resets to.
			audio.defaultPlaybackRate = next;
			audio.playbackRate = next;
		}
		publishMediaPositionState();
	}

	// Records the progress already carried by a freshly fetched episode list
	// (the episode endpoints include `position_seconds`/`listen`/`listened_at`),
	// so resume works without a per-play request. Live entries from this
	// session are kept.
	function seedProgress(episodes: Episode[]) {
		for (const episode of episodes) {
			seedChannelSpeed(episode);
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
			currentEpisode: currentEpisode.value,
			seedOrder: seedOrder.value,
			shuffle: shuffle.value,
			repeat: repeat.value,
			channelSpeedBySlug: channelSpeedBySlug.value
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
			seedOrder.value = stored.seedOrder;
			playStack.value = stored.playStack;
			currentEpisode.value = stored.currentEpisode;
			shuffle.value = stored.shuffle;
			repeat.value = stored.repeat;
			channelSpeedBySlug.value = stored.channelSpeedBySlug;
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

	function getMediaSession(): MediaSession | null {
		if (typeof navigator === 'undefined' || !('mediaSession' in navigator)) return null;
		return navigator.mediaSession ?? null;
	}

	function runMediaSessionAction(generation: number, action: () => void | Promise<void>) {
		if (!mediaSessionRegistered || generation !== mediaSessionGeneration) return;
		try {
			Promise.resolve(action()).catch(() => {});
		} catch {
			// A system gesture must never break ordinary in-app playback.
		}
	}

	function usableMediaDuration(): number | null {
		const value = audio?.duration;
		return value != null && Number.isFinite(value) && value > 0 ? value : null;
	}

	function seekRelativeFromSystem(details: MediaSessionActionDetails, direction: 1 | -1) {
		if (!usableMediaDuration()) return;
		const requested = details.seekOffset;
		const offset =
			requested != null && Number.isFinite(requested) && requested > 0
				? requested
				: KEYBOARD_SEEK_STEP;
		seekRelative(direction * offset);
	}

	function seekAbsoluteFromSystem(details: MediaSessionActionDetails) {
		const max = usableMediaDuration();
		const requested = details.seekTime;
		if (max == null || requested == null || !Number.isFinite(requested)) return;
		seek(Math.min(Math.max(requested, 0), max));
	}

	function ensureMediaSession() {
		const session = getMediaSession();
		if (!session || mediaSessionRegistered) return;
		mediaSessionRegistered = true;
		const generation = ++mediaSessionGeneration;
		const handlers: Array<
			[MediaSessionAction, (details: MediaSessionActionDetails) => void | Promise<void>]
		> = [
			['play', () => (audio?.paused && currentEpisode.value ? togglePlay() : undefined)],
			['pause', () => (!audio?.paused ? pause() : undefined)],
			['nexttrack', () => (upNext.value.length > 0 ? skipNext() : undefined)],
			[
				'previoustrack',
				() =>
					currentEpisode.value && (currentTime.value > 3 || playStack.value.length > 0)
						? playPrevious()
						: undefined
			],
			['seekforward', (details) => seekRelativeFromSystem(details, 1)],
			['seekbackward', (details) => seekRelativeFromSystem(details, -1)],
			['seekto', seekAbsoluteFromSystem]
		];
		for (const [action, handler] of handlers) {
			try {
				session.setActionHandler(action, (details) =>
					runMediaSessionAction(generation, () => handler(details))
				);
			} catch {
				// Browsers expose different action subsets; keep the rest available.
			}
		}
		if (currentEpisode.value) publishMediaMetadata(currentEpisode.value);
	}

	function publishMediaMetadata(episode: Episode) {
		const session = getMediaSession();
		if (!session || typeof MediaMetadata === 'undefined') return;
		const base: MediaMetadataInit = { title: episode.title, artist: episode.channel_title };
		const image = episode.image?.trim();
		try {
			session.metadata = new MediaMetadata(image ? { ...base, artwork: [{ src: image }] } : base);
		} catch {
			try {
				session.metadata = new MediaMetadata(base);
			} catch {
				// Text metadata is optional when the browser rejects construction.
			}
		}
	}

	function publishMediaPlaybackState(state?: MediaSessionPlaybackState) {
		const session = getMediaSession();
		if (!session) return;
		try {
			session.playbackState =
				state ?? (stopped.value ? 'none' : playing.value ? 'playing' : 'paused');
		} catch {
			// Playback continues when the browser rejects a state transition.
		}
	}

	function publishMediaPositionState() {
		const session = getMediaSession();
		if (!session || typeof session.setPositionState !== 'function') return;
		const mediaDuration = usableMediaDuration();
		const position = audio?.currentTime;
		const playbackRate = audio?.playbackRate ?? speed.value;
		if (
			mediaDuration == null ||
			position == null ||
			!Number.isFinite(position) ||
			!Number.isFinite(playbackRate) ||
			playbackRate <= 0
		)
			return;
		try {
			session.setPositionState({
				duration: mediaDuration,
				position: Math.min(Math.max(position, 0), mediaDuration),
				playbackRate
			});
		} catch {
			// Invalid/transitional values must not interrupt audio playback.
		}
	}

	function ensureAudio(): HTMLAudioElement | null {
		if (typeof window === 'undefined') return null;
		ensureMediaSession();
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
		if (audio) {
			const target = sponsorBlockSkipTarget(
				audio.currentTime,
				activeSponsorBlockSegments(currentEpisode.value)
			);
			if (target !== audio.currentTime) audio.currentTime = target;
			currentTime.value = target;
		}
		tryResumeSeek();
		publishMediaPositionState();
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
		publishMediaPositionState();
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
		if (
			isFinite(el.duration) &&
			el.duration > 0 &&
			resumeTarget >= el.duration * RESUME_DURATION_RATIO
		) {
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
		const target = sponsorBlockSkipTarget(resumeTarget, activeSponsorBlockSegments(current));
		el.currentTime = target;
		currentTime.value = target;
		publishMediaPositionState();
	}

	// Starts playback, arming the resume retry loop when one is pending.
	async function seekResumeOnPlay(el: HTMLAudioElement): Promise<void> {
		const current = currentEpisode.value;
		const pending = resumePending && current != null && current.id === resumeEpisodeId;
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
		stopped.value = false;
		publishMediaPlaybackState('playing');
		publishMediaPositionState();
	}

	function onPause() {
		playing.value = false;
		persistProgress();
		publishMediaPlaybackState(stopped.value ? 'none' : 'paused');
		publishMediaPositionState();
	}

	function onWaiting() {
		loading.value = true;
	}

	function onCanPlay() {
		loading.value = false;
	}

	function onEnded() {
		playing.value = false;
		publishMediaPlaybackState('none');
		publishMediaPositionState();
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
		// Never regress the position of a *just-finalized* listened episode: a late
		// flush (e.g. after `skipNext(markCurrent)` finalizes it at its
		// duration) must not overwrite the completion position with the live
		// playhead. Once playback restarts on the episode the marker is cleared
		// and live re-listens persist normally again.
		if (mark && pos < episode.position_seconds && finalizedEpisodeId === episode.id) return;
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
		// Finalize: until the user plays this episode again, no stale flush is
		// allowed to regress the completion position.
		finalizedEpisodeId = episode.id;
		currentEpisode.value = { ...episode };
		lastSavedPosition = position;
		lastSavedEpisodeId = episode.id;
		recordProgress(episode, {
			position_seconds: position,
			listen: true,
			listened_at: listenedAt
		});
		api
			.updateEpisodeProgress(episode.yt_id, {
				position_seconds: position,
				listened: true
			})
			.catch((err) => {
				console.error('Failed to save listened mark', err);
			});
		// Playlist lifecycle: an episode finished from the playlist source leaves
		// the playlist too. Fire-and-forget; a 404 (already removed by a racing
		// completion) is ignored and the next playlist read reconciles.
		if (queueSource.value === 'playlist') {
			usePlaylistStore()
				.remove(episode.id)
				.catch((err) => console.error('Failed to remove from playlist', err));
		}
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
		publishMediaMetadata(episode);
		publishMediaPlaybackState('paused');
		// Playback is (re)starting on this episode: any previous finalize no
		// longer protects its completion position from live saves.
		finalizedEpisodeId = null;
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
		// load() above resets the element's playbackRate to defaultPlaybackRate:
		// the channel speed must be (re)applied only after any reload, right
		// before playback starts, so the saved rate is actually audible and the
		// previous channel's rate can never leak into the new episode
		// (per-channel-playback-speed).
		applyChannelSpeed(episode);
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

	async function play(
		episode: Episode,
		list?: Episode[],
		opts?: { fromStart?: boolean; queueSource?: 'playlist' | 'list' }
	) {
		const fromStart = opts?.fromStart ?? false;
		// Re-seeded on every play: the queue source decides whether a finished
		// episode also leaves the playlist (playlist-capability).
		queueSource.value = opts?.queueSource ?? 'list';
		if (list) {
			const index = list.findIndex((e) => e.id === episode.id);
			upNext.value = index < 0 ? [] : list.slice(index + 1);
			// The authored seed for shuffle/repeat-all: the queue as authored,
			// independent of the consumption order. An active shuffle applies
			// to the freshly seeded queue (playback-modes).
			seedOrder.value = [...upNext.value];
			if (shuffle.value) {
				upNext.value = shuffledCopy(upNext.value);
			}
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

	// Toggles shuffle (playback-modes). Enabling builds a shuffled copy of the
	// current consumption order; disabling restores the authored order of the
	// remaining episodes from the seed. `upNext` is always replaced with a
	// fresh array, never reordered in place, so the authored seed stays intact.
	function toggleShuffle() {
		if (shuffle.value) {
			shuffle.value = false;
			upNext.value = seedOrder.value.filter((e) =>
				upNext.value.some((remaining) => remaining.id === e.id)
			);
		} else {
			shuffle.value = true;
			upNext.value = shuffledCopy(upNext.value);
		}
		persistQueue();
	}

	// Repeat control: none → all → one → none (playback-modes). Returns the
	// new mode so the persistent bar can reflect it.
	function cycleRepeat(): RepeatMode {
		repeat.value = repeat.value === 'none' ? 'all' : repeat.value === 'all' ? 'one' : 'none';
		persistQueue();
		return repeat.value;
	}

	// Mobile expanded view's combined shuffle/repeat control
	// (expand-mobile-player-view): advances through normal -> repeat -> shuffle
	// -> normal, setting shuffle and repeat directly to land exactly on the
	// selected state's underlying values. Reuses the current mode's closest
	// mapping so pressing it always lands on an exact, reachable state.
	function cycleMobilePlaybackMode(): MobilePlaybackMode {
		const current = closestMobilePlaybackMode(shuffle.value, repeat.value);
		const next = nextMobilePlaybackMode(current);
		if (next === 'normal') {
			shuffle.value = false;
			repeat.value = 'none';
		} else if (next === 'repeat') {
			shuffle.value = false;
			repeat.value = 'all';
		} else {
			shuffle.value = true;
			repeat.value = 'none';
		}
		persistQueue();
		return next;
	}

	async function advance() {
		const finished = currentEpisode.value;
		if (repeat.value === 'one' && finished) {
			// Repeat-one: replay the finished episode from the start; the queue
			// is not consumed and the episode does not enter history
			// (playback-modes).
			await play(finished, undefined, { fromStart: true });
			return;
		}
		const next = upNext.value.shift();
		if (next) {
			if (finished) pushToPlayStack(finished);
			// The next queued episode resumes from its saved position when it
			// has one, just like a fresh `play()` (playback-progress).
			await armResume(next, 'advance');
			await loadEpisode(next);
			persistQueue();
			return;
		}
		if (repeat.value === 'all' && seedOrder.value.length > 0) {
			// Repeat-all: rebuild the queue from the seeded source — re-shuffled
			// on every cycle when shuffle is active — and start the new cycle
			// (playback-modes).
			upNext.value = shuffle.value ? shuffledCopy(seedOrder.value) : [...seedOrder.value];
			const replayed = upNext.value.shift();
			if (replayed) {
				if (finished) pushToPlayStack(finished);
				await armResume(replayed, 'advance');
				await loadEpisode(replayed);
				persistQueue();
				return;
			}
		}
		// End of the queue with no mode wanting a replay: halt and keep this
		// episode (it was just completed), without the public stop's reset
		// behavior. The seeded source is gone with the queue.
		haltPlayback();
		upNext.value = [];
		seedOrder.value = [];
		persistQueue();
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
			publishMediaPositionState();
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
		// A user removal also leaves the authored seed so nothing (shuffle
		// restore or repeat-all rebuild) can resurrect it.
		seedOrder.value = seedOrder.value.filter((e) => e.id !== episodeId);
		persistQueue();
	}

	function clearQueue() {
		upNext.value = [];
		seedOrder.value = [];
		persistQueue();
	}

	function syncPlaylistOrder(episodes: Episode[]) {
		if (queueSource.value !== 'playlist' || currentEpisode.value == null) return;
		const currentIndex = episodes.findIndex((episode) => episode.id === currentEpisode.value?.id);
		if (currentIndex < 0) return;

		const nextSeed = episodes.slice(currentIndex + 1);
		if (shuffle.value) {
			const nextIds = new Set(nextSeed.map((episode) => episode.id));
			const retained = upNext.value.filter((episode) => nextIds.has(episode.id));
			const retainedIds = new Set(retained.map((episode) => episode.id));
			const added = nextSeed.filter((episode) => !retainedIds.has(episode.id));
			upNext.value = [...retained, ...shuffledCopy(added)];
		} else {
			upNext.value = [...nextSeed];
		}
		seedOrder.value = [...nextSeed];
		persistQueue();
	}

	async function togglePlay() {
		const el = ensureAudio();
		if (!el || !currentEpisode.value) return;
		if (el.paused) {
			const wasStopped = stopped.value;
			stopped.value = false;
			// Playback is restarting: a previous finalize no longer protects
			// the episode's completion position from live saves.
			finalizedEpisodeId = null;
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
			// load() above (and every real load()) resets playbackRate: re-apply
			// the channel's saved speed after any reload and before play, so a
			// restored-queue restart starts at the right rate
			// (per-channel-playback-speed).
			applyChannelSpeed(currentEpisode.value);
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

	// Halts playback and keeps the saved position (no reset). Used by the
	// internal end-of-queue stop and the session-teardown stop.
	function haltPlayback() {
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
		publishMediaPlaybackState('none');
		publishMediaPositionState();
	}

	// Resets the saved start point of an episode to 0, keeping its listened
	// mark unchanged. A user gesture: bypasses the "never regress a listened
	// position" rule and always writes.
	function resetPosition(episode: Episode) {
		const recorded = effectiveProgress(episode);
		if (recorded.position_seconds === 0) return; // already at the start
		finalizedEpisodeId = null;
		lastSavedPosition = 0;
		lastSavedEpisodeId = episode.id;
		recordProgress(episode, {
			position_seconds: 0,
			listen: recorded.listen,
			listened_at: recorded.listened_at ?? null
		});
		// Keep the shared current-episode view in sync when the reset targets
		// the current episode (card stop on the current, non-reproducing
		// episode), so the bar/card reflect the cleared position immediately.
		if (currentEpisode.value?.id === episode.id) {
			currentEpisode.value = { ...currentEpisode.value, position_seconds: 0 };
		}
		trace('reset position to zero', episode.yt_id);
		api
			.updateEpisodeProgress(episode.yt_id, {
				position_seconds: 0,
				listened: recorded.listen
			})
			.catch((err) => {
				console.error('Failed to reset playback progress', err);
			});
	}

	// Public stop control. Callers pass the episode the button belongs to:
	// - the persistent bar calls stop() with no target: a pure stop that halts
	//   playback when reproducing and never touches any saved position;
	// - the cards call stop(episode): when that episode is reproducing (it is
	//   the current one and playing) it halts keeping the position; otherwise
	//   (a non-current card, or the current episode stopped or paused) it
	//   resets the episode's saved position to 0 (fix-stop-reset-scope).
	function stop(target?: Episode) {
		const targetEpisode = target ?? currentEpisode.value;
		if (!targetEpisode) return;
		const isCurrentTarget = target == null || targetEpisode.id === currentEpisode.value?.id;
		const el = audio;
		const reproducing = isCurrentTarget && !stopped.value && el != null && !el.paused;
		if (reproducing) {
			haltPlayback();
			return;
		}
		if (target != null) {
			resetPosition(targetEpisode);
			return;
		}
		// Persistent-bar stop: converge to the stopped state without touching
		// any saved position.
		stopped.value = true;
		playing.value = false;
		currentTime.value = 0;
		if (el) {
			el.pause();
			el.currentTime = 0;
		}
		publishMediaPlaybackState('none');
		publishMediaPositionState();
	}

	function teardownNativeMedia() {
		persistProgress();
		finishResume();
		playing.value = false;
		loading.value = false;
		stopped.value = true;
		currentTime.value = 0;
		duration.value = 0;
		if (audio) {
			audio.pause();
			audio.removeAttribute('src');
			audio.load();
		}
		const session = getMediaSession();
		mediaSessionRegistered = false;
		mediaSessionGeneration += 1;
		if (session) {
			publishMediaPlaybackState('none');
			try {
				session.metadata = null;
			} catch {
				// Metadata cleanup is best effort on partial implementations.
			}
			for (const action of MEDIA_SESSION_ACTIONS) {
				try {
					session.setActionHandler(action, null);
				} catch {
					// Ignore actions the browser never supported.
				}
			}
		}
	}

	function seek(seconds: number) {
		if (!audio) return;
		const target = sponsorBlockSkipTarget(
			seconds,
			activeSponsorBlockSegments(currentEpisode.value)
		);
		audio.currentTime = target;
		currentTime.value = target;
		publishMediaPositionState();
	}

	// Keyboard shortcut: seek ±15s clamped to the episode bounds. Persisted by
	// the existing throttled/event-driven saves, exactly like scrubber seeks.
	function seekRelative(delta: number) {
		if (!audio || !currentEpisode.value) return;
		const max = isFinite(audio.duration) && audio.duration > 0 ? audio.duration : 0;
		const requested = Math.min(Math.max(audio.currentTime + delta, 0), max);
		const next = sponsorBlockSkipTarget(
			requested,
			activeSponsorBlockSegments(currentEpisode.value)
		);
		audio.currentTime = next;
		currentTime.value = next;
		publishMediaPositionState();
	}

	function applySponsorBlockSnapshot(episode: Episode) {
		if (currentEpisode.value?.id !== episode.id) return;
		const enabled = episode.sponsorblock_enabled === true;
		const segments = enabled ? (episode.sponsorblock_segments ?? []) : [];
		const hash = enabled ? (episode.sponsorblock_hash ?? null) : null;
		if (
			currentEpisode.value.sponsorblock_enabled === enabled &&
			currentEpisode.value.sponsorblock_hash === hash &&
			JSON.stringify(currentEpisode.value.sponsorblock_segments ?? []) === JSON.stringify(segments)
		)
			return;
		currentEpisode.value = {
			...currentEpisode.value,
			sponsorblock_enabled: enabled,
			sponsorblock_segments: segments,
			sponsorblock_hash: hash
		};
		persistQueue();
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

	// Applies the rate immediately and, with an episode loaded, makes the
	// value the channel's saved speed: the local per-channel map is upserted
	// and the change is persisted server-side fire-and-forget (mirroring
	// persistProgress), so every other episode of the channel starts here
	// (per-channel-playback-speed).
	function setSpeed(value: number) {
		const clamped = Math.min(SPEED_MAX, Math.max(SPEED_MIN, value));
		const next = Math.round(clamped * 100) / 100;
		speed.value = next;
		if (audio) audio.playbackRate = next;
		publishMediaPositionState();
		const episode = currentEpisode.value;
		if (episode && episode.channel_slug) {
			channelSpeedBySlug.value = {
				...channelSpeedBySlug.value,
				[episode.channel_slug]: next
			};
			api.setChannelPlaybackSpeed(episode.channel_slug, next).catch((err) => {
				console.error('Failed to save channel playback speed', err);
			});
		}
		// Keep the persisted queue's speed map current so a reload restores
		// the latest per-channel values.
		persistQueue();
	}

	const progress = computed(() =>
		duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0
	);

	const currentLabel = computed(() => {
		if (duration.value > 0) {
			const hours = Math.floor(currentTime.value / 3600);
			const minutes = Math.floor((currentTime.value % 3600) / 60);
			const seconds = Math.floor(currentTime.value % 60);
			const minuteSeconds = `${minutes}:${String(seconds).padStart(2, '0')}`;
			return hours > 0
				? `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
				: minuteSeconds;
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

	// Remaining time label for the mobile expanded now-playing view
	// (expand-mobile-player-view): duration minus elapsed, formatted like
	// `currentLabel`/`durationLabel` (M:SS / MM:SS / H:MM:SS).
	const remainingLabel = computed(() => {
		const total =
			duration.value > 0 ? duration.value : parseDurationSeconds(currentEpisode.value?.duration);
		if (total == null || total <= 0) return '0:00';
		const remaining = Math.max(0, total - currentTime.value);
		const hours = Math.floor(remaining / 3600);
		const minutes = Math.floor((remaining % 3600) / 60);
		const seconds = Math.floor(remaining % 60);
		const minuteSeconds = `${minutes}:${String(seconds).padStart(2, '0')}`;
		return hours > 0
			? `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
			: minuteSeconds;
	});

	function isCurrent(episode: Episode) {
		return currentEpisode.value != null && currentEpisode.value.id === episode.id;
	}

	const mobilePlaybackMode = computed(() => closestMobilePlaybackMode(shuffle.value, repeat.value));

	return {
		mobilePlaybackMode,
		cycleMobilePlaybackMode,
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
		queueSource,
		shuffle,
		repeat,
		progress,
		currentLabel,
		durationLabel,
		remainingLabel,
		play,
		advance,
		skipNext,
		playPrevious,
		toggleShuffle,
		cycleRepeat,
		removeFromQueue,
		clearQueue,
		syncPlaylistOrder,
		togglePlay,
		pause,
		stop,
		halt: haltPlayback,
		teardownNativeMedia,
		seek,
		seekRelative,
		episodeWithProgress,
		seedProgress,
		applyProgress,
		applySponsorBlockSnapshot,
		setVolume,
		toggleMute,
		setSpeed,
		isCurrent
	};
});
