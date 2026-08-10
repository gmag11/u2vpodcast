import { writable } from 'svelte/store';

export const notification = writable<{
	message: string;
	colorName: string;
	borderColor?: string;
	textTopColor?: string;
	textBottomColor?: string;
}>({
	message: '',
	colorName: '',
	borderColor: '',
	textTopColor: '',
	textBottomColor: ''
});
