import type { Channel, ConfigResponse, Episode, LoginRequestBody, Response, User } from '@/types';

export const baseEndpoint: string = import.meta.env.DEV ? 'http://localhost:6996' : '';

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

	async getEpisodes(channelId: number) {
		return request<Array<Episode>>(`/api/1.0/channels/${channelId}/episodes/`);
	},

	async getConfig() {
		return request<ConfigResponse>('/api/1.0/config/');
	}
};
