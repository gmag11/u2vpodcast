<script setup lang="ts">
	import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
	import { useI18n } from 'vue-i18n';
	import { PhGlobe, PhList, PhMoon, PhSignOut, PhSun, PhUserCircle } from '@phosphor-icons/vue';
	import {
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuPortal,
		DropdownMenuRoot,
		DropdownMenuTrigger
	} from 'radix-vue';
	import { useRoute, useRouter } from 'vue-router';
	import i18n, { AVAILABLE_LOCALES } from '@/i18n';
	import { useAuthStore } from '@/stores/auth';
	import { useLocaleStore } from '@/stores/locale';
	import { useThemeStore } from '@/stores/theme';

	const route = useRoute();
	const router = useRouter();
	const auth = useAuthStore();
	const theme = useThemeStore();
	const locale = useLocaleStore();
	const { t } = useI18n();

	const drawerOpen = ref(false);

	const currentLanguageName = computed(() =>
		i18n.global.locale.value === 'es' ? t('languages.es') : t('languages.en')
	);

	function closeOverlays() {
		drawerOpen.value = false;
	}

	function onKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') closeOverlays();
	}

	async function handleLogout() {
		await auth.logout();
		router.push({ name: 'login' });
	}

	onMounted(() => window.addEventListener('keydown', onKeydown));
	onUnmounted(() => window.removeEventListener('keydown', onKeydown));
	watch(() => route.fullPath, closeOverlays);
</script>

<template>
	<nav
		class="fixed top-0 z-50 w-full border-b border-outline bg-surface/70 shadow-[0_4px_20px_var(--glow)] backdrop-blur-xl"
	>
		<div class="mx-auto flex h-20 max-w-[1440px] items-center justify-between px-5 md:px-16">
			<div class="flex items-center gap-6">
				<div class="flex items-center gap-3">
					<span
						class="flex h-9 w-9 items-center justify-center rounded-full bg-primary-500/10 text-xl text-primary-500"
					>
						<slot name="brand-icon" />
					</span>
					<span
						class="hidden font-display text-2xl font-semibold tracking-tight text-text lg:inline"
					>
						U2V<span class="text-primary-500">Podcast</span>
					</span>
				</div>

				<div class="hidden items-center gap-1 md:flex">
					<RouterLink
						to="/"
						class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
						:class="
							route.name === 'channels'
								? 'bg-accent-600 text-white'
								: 'text-text-muted hover:text-text'
						"
					>
						{{ $t('header.channels') }}
					</RouterLink>
					<RouterLink
						to="/history"
						class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
						:class="
							route.name === 'history'
								? 'bg-accent-600 text-white'
								: 'text-text-muted hover:text-text'
						"
					>
						{{ $t('header.history') }}
					</RouterLink>
					<RouterLink
						to="/playlist"
						class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
						:class="
							route.name === 'playlist'
								? 'bg-accent-600 text-white'
								: 'text-text-muted hover:text-text'
						"
					>
						{{ $t('playlist.title') }}
					</RouterLink>
				</div>
			</div>

			<div class="flex items-center gap-2 lg:gap-4">
				<slot name="actions" />
				<DropdownMenuRoot>
					<DropdownMenuTrigger
						class="flex items-center gap-1.5 rounded-lg p-2 text-text-muted transition-colors hover:text-text"
						:aria-label="$t('header.language')"
					>
						<PhGlobe class="h-5 w-5" weight="regular" />
						<span class="hidden text-sm font-medium md:inline">{{ currentLanguageName }}</span>
					</DropdownMenuTrigger>
					<DropdownMenuPortal>
						<DropdownMenuContent
							class="z-[70] min-w-36 rounded-lg border border-outline bg-surface-card p-1 shadow-card"
							:side-offset="8"
							align="end"
						>
							<DropdownMenuItem
								v-for="code in AVAILABLE_LOCALES"
								:key="code"
								class="flex cursor-pointer items-center justify-between gap-3 rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
								:class="
									locale.locale === code
										? 'bg-accent-600 text-white'
										: 'text-text-muted hover:text-text'
								"
								@select="locale.apply(code)"
							>
								<span>{{ t(`languages.${code}`) }}</span>
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenuPortal>
				</DropdownMenuRoot>
				<button
					type="button"
					:aria-label="$t('common.toggleTheme')"
					class="rounded-lg p-2 text-text-muted transition-colors hover:text-text"
					@click="theme.toggle()"
				>
					<PhSun v-if="theme.theme === 'dark'" class="h-5 w-5" weight="regular" />
					<PhMoon v-else class="h-5 w-5" weight="regular" />
				</button>
				<button
					type="button"
					class="rounded-lg p-2 text-text-muted transition-colors hover:text-text md:hidden"
					:aria-label="$t('common.menuOpen')"
					:aria-expanded="drawerOpen"
					@click="drawerOpen = !drawerOpen"
				>
					<PhList class="h-6 w-6" weight="bold" />
				</button>
				<div class="hidden items-center gap-4 border-l border-outline pl-6 md:flex">
					<PhUserCircle class="h-10 w-10 text-text-muted" weight="fill" />
					<button
						type="button"
						class="flex items-center gap-2 rounded-lg border border-outline px-4 py-2 text-sm font-medium text-text-muted transition-colors hover:text-text"
						@click="handleLogout"
					>
						{{ $t('header.logout') }}
						<PhSignOut class="h-4 w-4" weight="regular" />
					</button>
				</div>
			</div>
		</div>
	</nav>

	<div
		v-if="drawerOpen"
		class="fixed inset-0 z-[60] bg-black/60 backdrop-blur-sm md:hidden"
		@click="drawerOpen = false"
	></div>
	<aside
		v-if="drawerOpen"
		class="fixed right-0 top-0 z-[70] flex h-full w-72 flex-col border-l border-outline bg-surface-card p-5 shadow-card md:hidden"
		role="dialog"
		:aria-label="$t('common.navigation')"
	>
		<div class="flex items-center gap-3 border-b border-outline pb-4">
			<PhUserCircle class="h-10 w-10 shrink-0 text-text-muted" weight="fill" />
			<div class="min-w-0">
				<p class="truncate text-sm font-semibold text-text">
					{{ auth.user?.name ?? $t('header.userFallback') }}
				</p>
				<p class="text-xs text-text-muted">{{ auth.user?.role }}</p>
			</div>
		</div>

		<nav class="mt-4 flex flex-col gap-1">
			<RouterLink
				to="/"
				class="rounded-md px-3 py-2 text-sm font-medium transition-colors"
				:class="
					route.name === 'channels' ? 'bg-accent-600 text-white' : 'text-text-muted hover:text-text'
				"
				@click="drawerOpen = false"
			>
				{{ $t('header.channels') }}
			</RouterLink>
			<RouterLink
				to="/history"
				class="rounded-md px-3 py-2 text-sm font-medium transition-colors"
				:class="
					route.name === 'history' ? 'bg-accent-600 text-white' : 'text-text-muted hover:text-text'
				"
				@click="drawerOpen = false"
			>
				{{ $t('header.history') }}
			</RouterLink>
			<RouterLink
				to="/playlist"
				class="rounded-md px-3 py-2 text-sm font-medium transition-colors"
				:class="
					route.name === 'playlist' ? 'bg-accent-600 text-white' : 'text-text-muted hover:text-text'
				"
				@click="drawerOpen = false"
			>
				{{ $t('playlist.title') }}
			</RouterLink>
		</nav>

		<div class="mt-auto flex flex-col gap-3 border-t border-outline pt-4">
			<button
				type="button"
				class="flex w-full items-center justify-center gap-2 rounded-lg border border-outline px-4 py-2 text-sm font-medium text-text-muted transition-colors hover:text-text"
				@click="handleLogout"
			>
				{{ $t('header.logout') }}
				<PhSignOut class="h-4 w-4" weight="regular" />
			</button>
		</div>
	</aside>
</template>
