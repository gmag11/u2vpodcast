import type { Episode } from '@/types';

export type RepeatMode = 'none' | 'all' | 'one';

export interface QueuePayload {
	upNext: Episode[];
	playStack: Episode[];
	currentEpisode: Episode | null;
	// Playback modes (playback-modes): the authored seed used to (re)build the
	// shuffled/repeat-all consumption order, plus the mode flags. Optional so
	// payloads written by earlier versions still load; loadQueue() normalizes
	// the defaults.
	seedOrder?: Episode[];
	shuffle?: boolean;
	repeat?: RepeatMode;
	// Per-channel playback speeds (per-channel-playback-speed): lets a reloaded
	// session start episodes at the right rate even though restored episodes
	// carry no payload fields. Optional like the modes.
	channelSpeedBySlug?: Record<string, number>;
}

/** `QueuePayload` as normalized by `loadQueue()`: defaults are resolved. */
export interface ResolvedQueuePayload extends QueuePayload {
	seedOrder: Episode[];
	shuffle: boolean;
	repeat: RepeatMode;
	channelSpeedBySlug: Record<string, number>;
}

const STORAGE_KEY = 'u2vpodcast.up-next.v1';

/**
 * Persists the up-next queue and playback history. Best effort: storage is
 * unavailable (private mode, quota) and failures are swallowed, never raised.
 */
export function saveQueue(payload: QueuePayload): void {
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(payload));
	} catch {
		// ignore — the queue just does not survive the reload
	}
}

/**
 * Loads a previously persisted queue, returning `null` when nothing is stored
 * or the payload is unreadable/malformed so callers can fall back to empty.
 * Payloads written by earlier versions (without `currentEpisode`, `seedOrder`
 * or the playback modes) still load; missing fields are normalized to their
 * defaults.
 */
export function loadQueue(): ResolvedQueuePayload | null {
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return null;
		const parsed: unknown = JSON.parse(raw);
		if (
			parsed == null ||
			typeof parsed !== 'object' ||
			!Array.isArray((parsed as QueuePayload).upNext) ||
			!Array.isArray((parsed as QueuePayload).playStack)
		) {
			return null;
		}
		const payload = parsed as QueuePayload;
		return {
			upNext: payload.upNext,
			playStack: payload.playStack,
			currentEpisode: payload.currentEpisode ?? null,
			// The authored seed for shuffle/repeat-all. Legacy payloads written
			// before playback-modes lack it; the stored queue is then the best
			// available source.
			seedOrder: Array.isArray(payload.seedOrder) ? payload.seedOrder : [...payload.upNext],
			shuffle: payload.shuffle ?? false,
			repeat: payload.repeat === 'all' || payload.repeat === 'one' ? payload.repeat : 'none',
			// Per-channel speeds from newer payloads; legacy payloads yield an
			// empty map and episodes fall back to 1.0 until a fetch seeds them.
			channelSpeedBySlug: payload.channelSpeedBySlug ?? {}
		};
	} catch {
		return null;
	}
}
