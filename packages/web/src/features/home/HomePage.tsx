import { Navigate } from '@solidjs/router';
import { Match, Show, Switch } from 'solid-js';
import { token } from '../../api/session';
import { t } from '../../i18n';
import { useCurrentUser, useSignOut } from '../auth/queries';

export default function HomePage() {
	const user = useCurrentUser();
	const signOut = useSignOut();

	return (
		<Show when={token()} fallback={<Navigate href="/sign-in" />}>
			<main class="mx-auto flex min-h-screen max-w-sm flex-col justify-center gap-4 px-4">
				<Switch>
					<Match when={user.isError}>
						<p role="alert">{t('home.loadFailed')}</p>
					</Match>
					<Match when={user.data}>
						{(data) => <p>{t('home.signedInAs', { email: data().email })}</p>}
					</Match>
					<Match when={true}>
						<p>{t('home.loading')}</p>
					</Match>
				</Switch>
				<button class="rounded border px-3 py-2" type="button" onClick={() => void signOut()}>
					{t('home.signOut')}
				</button>
			</main>
		</Show>
	);
}
