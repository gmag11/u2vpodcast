export interface User {
	id: number;
	name: string;
	role: string;
	active: boolean;
}

export interface Channel {
	id: number;
	url: string;
	title: string;
	slug: string;
	active: boolean;
	description: string;
	image: string;
	first: Date;
	max: number;
	created_at: Date;
	updated_at: Date;
	last_date: string | null;
	last_sync_at: string | null;
	last_sync_ok: boolean | null;
	last_sync_error: string | null;
}

export interface Episode {
	id: number;
	channel_id: number;
	channel_slug: string;
	channel_title: string;
	title: string;
	description: string;
	yt_id: string;
	webpage_url: string;
	published_at: Date;
	duration: string;
	image: string;
	listen: boolean;
	created_at: Date;
	updated_at: Date;
}

export interface Response {
	status: boolean;
	status_code: number;
	message: string;
	user: User | null;
	data: Channel | Array<Channel> | Episode | Array<Episode> | null;
}

export interface ConfigResponse {
	data?: {
		per_page?: number;
	};
}

export interface LoginRequestBody {
	username: string;
	password: string;
}
