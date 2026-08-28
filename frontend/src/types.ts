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
	position_seconds: number;
	listened_at: string | null;
	favorite: boolean;
	sponsorblock_segments?: SponsorBlockSegment[];
	sponsorblock_hash?: string | null;
	created_at: Date;
	updated_at: Date;
}

export interface SponsorBlockSegment {
	start: number;
	end: number;
}

export interface EpisodeProgress {
	id: number;
	yt_id: string;
	position_seconds: number;
	listen: boolean;
	listened_at: string | null;
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
