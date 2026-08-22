<script setup lang="ts">
	import { ref } from 'vue';
	import { useI18n } from 'vue-i18n';
	import { useRoute, useRouter } from 'vue-router';
	import { PhLockKey, PhUser } from '@phosphor-icons/vue';
	import { api } from '@/lib/api/client';
	import { useAuthStore } from '@/stores/auth';
	import { useLoadingStore } from '@/stores/loading';
	import { useNotificationStore } from '@/stores/notification';
	import AppButton from '@/components/AppButton.vue';
	import AppInput from '@/components/AppInput.vue';

	const router = useRouter();
	const route = useRoute();
	const auth = useAuthStore();
	const loading = useLoadingStore();
	const notification = useNotificationStore();
	const { t } = useI18n();

	const username = ref('');
	const password = ref('');
	const error = ref('');

	async function handleLogin() {
		error.value = '';
		loading.start(t('auth.loggingIn'));
		try {
			const result = await api.login({ username: username.value, password: password.value });
			if (result.ok && result.user) {
				auth.setUser(result.user);
				notification.show(t('auth.success'), 'success');
				const next = (route.query.next as string) || '/';
				router.push(next);
			} else {
				error.value = t('auth.invalidCredentials');
			}
		} catch (err) {
			console.error(err);
			error.value = t('auth.unexpectedError');
		} finally {
			loading.stop();
		}
	}
</script>

<template>
	<div
		class="flex min-h-screen items-center justify-center p-4"
		style="
			background: radial-gradient(circle at 10% 20%, rgb(28, 38, 51) 0%, rgb(18, 24, 33) 90%);
			background-size: cover;
			background-attachment: fixed;
		"
	>
		<main
			class="w-full max-w-md rounded-2xl border border-white/5 bg-surface-card/40 p-8 shadow-[0_8px_32px_rgba(0,0,0,0.37)] backdrop-blur-xl sm:p-10"
		>
			<header class="mb-8 flex flex-col items-center">
				<div class="mb-6 flex items-center gap-2">
					<svg
						class="h-6 w-6 text-primary-500"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						viewbox="0 0 24 24"
						xmlns="http://www.w3.org/2000/svg"
					>
						<path
							d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"
							stroke-linecap="round"
							stroke-linejoin="round"
						></path>
					</svg>
					<span class="text-xl font-semibold tracking-wide text-white">U2VPodcast</span>
				</div>
				<h1 class="text-3xl font-bold text-gray-100">{{ $t('auth.title') }}</h1>
			</header>

			<form class="space-y-5" @submit.prevent="handleLogin">
				<p v-if="error" class="text-center text-sm text-rose-600">{{ error }}</p>

				<div>
					<AppInput
						id="username"
						v-model="username"
						:placeholder="$t('auth.username')"
						type="text"
						leading-icon
						required
					>
						<template #icon>
							<PhUser class="h-5 w-5 text-gray-400" weight="regular" />
						</template>
					</AppInput>
				</div>

				<div>
					<AppInput
						id="password"
						v-model="password"
						:placeholder="$t('auth.password')"
						type="password"
						leading-icon
						required
					>
						<template #icon>
							<PhLockKey class="h-5 w-5 text-gray-400" weight="regular" />
						</template>
					</AppInput>
				</div>

				<div class="pt-2">
					<AppButton type="submit" class="w-full py-3" variant="primary">
						{{ $t('auth.submit') }}
					</AppButton>
				</div>
			</form>
		</main>
	</div>
</template>
