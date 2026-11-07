import { beforeEach, describe, expect, it } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import router from '@/router';
import { useAuthStore } from '@/stores/auth';

const admin = { id: 1, name: 'admin', role: 'admin', active: true };

describe('router auth guards', () => {
	beforeEach(async () => {
		setActivePinia(createPinia());
		useAuthStore().setUser(null);
		// Anchor on a public route so each test starts from a different route
		// than the one it navigates to (re-navigating to the current route is a
		// no-op and would skip the guard).
		await router.replace('/share/reset');
	});

	it('redirects an anonymous visitor from a protected route to login', async () => {
		await router.push('/history');
		expect(router.currentRoute.value.name).toBe('login');
		expect(router.currentRoute.value.query.next).toBe('/history');
	});

	it('allows an anonymous visitor to open a share link', async () => {
		await router.push('/share/abc.def');
		expect(router.currentRoute.value.name).toBe('share');
	});

	it('does not redirect an authenticated visitor away from a share link', async () => {
		useAuthStore().setUser(admin);
		await router.push('/share/abc.def');
		expect(router.currentRoute.value.name).toBe('share');
	});

	it('still redirects an authenticated visitor away from the login screen', async () => {
		useAuthStore().setUser(admin);
		await router.push('/login');
		expect(router.currentRoute.value.name).toBe('channels');
	});
});
