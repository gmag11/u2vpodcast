import type { Episode } from '@/types';

export interface QueuePayload {
	upNext: Episode[];
	playStack: Episode[];
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
 */
export function loadQueue(): QueuePayload | null {
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
		return parsed as QueuePayload;
	} catch {
		return null;
	}
}