<script setup lang="ts">
	import { PhMoon, PhSignOut, PhSun, PhUserCircle } from '@phosphor-icons/vue';
	import { useRouter } from 'vue-router';
	import { useAuthStore } from '@/stores/auth';
	import { useThemeStore } from '@/stores/theme';

	const router = useRouter();
	const auth = useAuthStore();
	const theme = useThemeStore();

	async function handleLogout() {
		await auth.logout();
		router.push({ name: 'login' });
	}
</script>

<template>
	<nav
		class="fixed top-0 z-50 w-full border-b border-outline bg-surface/70 shadow-[0_4px_20px_var(--glow)] backdrop-blur-xl"
	>
		<div class="mx-auto flex h-20 max-w-[1440px] items-center justify-between px-5 md:px-16">
			<div class="flex items-center gap-3">
				<span
					class="flex h-9 w-9 items-center justify-center rounded-full bg-primary-500/10 text-xl text-primary-500"
				>
					<slot name="brand-icon" />
				</span>
				<span class="font-display text-2xl font-semibold tracking-tight text-text">
					Aura<span class="text-primary-500">Pod</span>
				</span>
			</div>

			<div v-if="$slots.search" class="mx-8 hidden max-w-md flex-1 md:flex">
				<slot name="search" />
			</div>

			<div class="flex items-center gap-6">
				<slot name="actions" />
				<button
					type="button"
					aria-label="Toggle theme"
					class="rounded-lg p-2 text-text-muted transition-colors hover:text-text"
					@click="theme.toggle()"
				>
					<PhSun v-if="theme.theme === 'dark'" class="h-5 w-5" weight="regular" />
					<PhMoon v-else class="h-5 w-5" weight="regular" />
				</button>
				<div class="flex items-center gap-4 border-l border-outline pl-6">
					<PhUserCircle class="h-10 w-10 text-text-muted" weight="fill" />
					<button
						type="button"
						class="flex items-center gap-2 rounded-lg border border-outline px-4 py-2 text-sm font-medium text-text-muted transition-colors hover:text-text"
						@click="handleLogout"
					>
						Logout
						<PhSignOut class="h-4 w-4" weight="regular" />
					</button>
				</div>
			</div>
		</div>
	</nav>
</template>
