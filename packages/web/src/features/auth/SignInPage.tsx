import { createMutation } from '@tanstack/solid-query';
import { Navigate, useNavigate } from '@solidjs/router';
import { Show } from 'solid-js';
import { token } from '../../api/session';
import { t } from '../../i18n';
import { SignInError, signIn } from './queries';

export default function SignInPage() {
	const navigate = useNavigate();
	const mutation = createMutation(() => ({
		mutationFn: (credentials: { email: string; password: string }) =>
			signIn(credentials.email, credentials.password),
		onSuccess: () => navigate('/', { replace: true })
	}));

	const onSubmit = (event: SubmitEvent) => {
		event.preventDefault();
		const form = new FormData(event.currentTarget as HTMLFormElement);
		mutation.mutate({
			email: String(form.get('email') ?? ''),
			password: String(form.get('password') ?? '')
		});
	};

	return (
		<Show when={!token()} fallback={<Navigate href="/" />}>
			<main class="mx-auto flex min-h-screen max-w-sm flex-col justify-center gap-4 px-4">
				<h1 class="text-2xl font-semibold">{t('signIn.title')}</h1>
				<form class="flex flex-col gap-3" onSubmit={onSubmit}>
					<label class="flex flex-col gap-1 text-sm">
						{t('signIn.email')}
						<input
							class="rounded border px-3 py-2"
							name="email"
							type="email"
							autocomplete="username"
							required
						/>
					</label>
					<label class="flex flex-col gap-1 text-sm">
						{t('signIn.password')}
						<input
							class="rounded border px-3 py-2"
							name="password"
							type="password"
							autocomplete="current-password"
							required
						/>
					</label>
					<Show when={mutation.error}>
						{(error) => (
							<p role="alert" class="text-sm text-red-600">
								{error() instanceof SignInError &&
								(error() as SignInError).reason === 'invalid_credentials'
									? t('signIn.invalid')
									: t('signIn.unavailable')}
							</p>
						)}
					</Show>
					<button
						class="rounded bg-black px-3 py-2 text-white disabled:opacity-50"
						type="submit"
						disabled={mutation.isPending}
					>
						{mutation.isPending ? t('signIn.submitting') : t('signIn.submit')}
					</button>
				</form>
			</main>
		</Show>
	);
}
