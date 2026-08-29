import type {
	Channel,
	ConfigResponse,
	Episode,
	EpisodeProgress,
	LoginRequestBody,
	Response,
	User
} from '@/types';

export const baseEndpoint: string = '';

const jsonHeaders = {
	Accept: 'application/json',
	'Content-Type': 'application/json'
};

export interface ApiResult<T> {
	ok: boolean;
	data: T | null;
	user: User | null;
	status: boolean;
	message?: string;
}

async function request<T>(
	path: string,
	init: RequestInit = {},
	expectResponse: boolean = true
): Promise<ApiResult<T>> {
	const res = await fetch(`${baseEndpoint}${path}`, {
		...init,
		headers: {
			...jsonHeaders,
			...init.headers
		},
		credentials: 'include'
	});

	if (!expectResponse || res.status === 204) {
		return { ok: res.ok, data: null, user: null, status: res.ok };
	}

	const body: Response = await res.json();
	return {
		ok: res.ok,
		data: (body.data ?? null) as T | null,
		user: body.user ?? null,
		status: body.status,
		message: body.message
	};
}

export const api = {
	async login(body: LoginRequestBody) {
		const result = await request<User>('/api/1.0/login/', {
			method: 'POST',
			body: JSON.stringify(body)
		});
		if (result.ok && result.user == null && result.data != null) {
			result.user = result.data as unknown as User;
		}
		return result;
	},

	async logout() {
		return request<null>('/api/1.0/logout/', { method: 'GET' });
	},

	async getSession() {
		const result = await request<User>('/api/1.0/session/');
		if (result.ok && result.user == null && result.data != null) {
			result.user = result.data as unknown as User;
		}
		return result;
	},

	async getChannels() {
		return request<Array<Channel>>('/api/1.0/channels/');
	},

	async createChannel(channel: Partial<Channel>) {
		return request<Channel>('/api/1.0/channels/', {
			method: 'POST',
			body: JSON.stringify(channel)
		});
	},

	async updateChannel(slug: string, channel: Partial<Channel>) {
		return request<Channel>(`/api/1.0/channels/${slug}/`, {
			method: 'PUT',
			body: JSON.stringify(channel)
		});
	},

	async deleteChannel(slug: string) {
		return request<null>(`/api/1.0/channels/${slug}/`, { method: 'DELETE' });
	},

	async refreshChannel(slug: string) {
		return request<Channel>(`/api/1.0/channels/${slug}/update/`, { method: 'POST' });
	},

	async refreshChannelImage(slug: string) {
		return request<Channel>(`/api/1.0/channels/${slug}/image/`, { method: 'POST' });
	},

	async getEpisodes(channelId: number) {
		return request<Array<Episode>>(`/api/1.0/channels/${channelId}/episodes/`);
	},

	async getAllEpisodes() {
		return request<Array<Episode>>('/api/1.0/episodes/');
	},

	async getEpisodeProgress(ytId: string) {
		return request<EpisodeProgress>(`/api/1.0/episodes/${ytId}/progress/`);
	},

	async updateEpisodeProgress(ytId: string, body: { position_seconds: number; listened: boolean }) {
		// The endpoint answers 204 without a body; there is no data to unwrap.
		return request<null>(`/api/1.0/episodes/${ytId}/progress/`, {
			method: 'PUT',
			body: JSON.stringify(body)
		});
	},

	async getPlaylist() {
		return request<Array<Episode>>('/api/1.0/playlist/');
	},

	async setEpisodeFavorite(ytId: string, favorite: boolean) {
		// The endpoint answers 204 without a body; there is no data to unwrap.
		return request<null>(`/api/1.0/episodes/${ytId}/favorite/`, {
			method: 'PUT',
			body: JSON.stringify({ favorite })
		});
	},

	async setChannelPlaybackSpeed(slug: string, playbackSpeed: number) {
		// The endpoint answers 204 without a body; there is no data to unwrap.
		return request<null>(`/api/1.0/channels/${slug}/playback_speed/`, {
			method: 'PUT',
			body: JSON.stringify({ playback_speed: playbackSpeed })
		});
	},

	async refreshEpisodeSponsorBlock(ytId: string) {
		return request<Episode>(`/api/1.0/episodes/${ytId}/sponsorblock/refresh/`, {
			method: 'POST'
		});
	},

	async addEpisodeToPlaylist(episodeId: number) {
		return request<Episode>('/api/1.0/playlist/', {
			method: 'POST',
			body: JSON.stringify({ episode_id: episodeId })
		});
	},

	async removeEpisodeFromPlaylist(episodeId: number) {
		return request<null>(`/api/1.0/playlist/${episodeId}/`, { method: 'DELETE' });
	},

	async reorderPlaylist(episodeIds: number[]) {
		return request<null>('/api/1.0/playlist/reorder/', {
			method: 'PUT',
			body: JSON.stringify({ episode_ids: episodeIds })
		});
	},

	async getConfig() {
		return request<ConfigResponse>('/api/1.0/config/');
	}
};
