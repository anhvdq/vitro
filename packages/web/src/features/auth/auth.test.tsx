import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from '../../App';
import { clearToken, saveToken, token } from '../../api/session';
import { createTranslator } from '../../i18n';

const t = createTranslator('vi');
const PASSWORD = 'correct horse';
const VALID_TOKEN = 'tok-1';

type Call = { method: string; path: string; authorization: string | null };

/** A stand-in API that only honors VALID_TOKEN, like the real server's sessions. */
function fakeApi(options: { revokeOnMe?: boolean } = {}) {
	const calls: Call[] = [];
	const reply = (status: number, body?: unknown) =>
		new Response(body === undefined ? null : JSON.stringify(body), {
			status,
			headers: { 'content-type': 'application/json' }
		});
	vi.stubGlobal(
		'fetch',
		vi.fn(async (request: Request) => {
			const path = new URL(request.url).pathname;
			const authorization = request.headers.get('Authorization');
			calls.push({ method: request.method, path, authorization });
			if (path === '/auth/login') {
				const body = (await request.json()) as { password: string };
				return body.password === PASSWORD
					? reply(200, { data: { access_token: VALID_TOKEN, user: { id: 'u1', email: 'a@b.c' } } })
					: reply(401, { error: { code: 'UNAUTHORIZED' } });
			}
			if (path === '/auth/me') {
				return authorization === `Bearer ${VALID_TOKEN}` && !options.revokeOnMe
					? reply(200, { data: { id: 'u1', email: 'owner@example.com' } })
					: reply(401, { error: { code: 'UNAUTHORIZED' } });
			}
			if (path === '/auth/logout') return reply(204);
			return reply(404);
		})
	);
	return calls;
}

function openAt(path: string) {
	window.history.pushState({}, '', path);
	render(() => <App />);
}

beforeEach(() => {
	sessionStorage.clear();
	clearToken();
});

afterEach(() => {
	cleanup();
	vi.unstubAllGlobals();
});

describe('sign in', () => {
	it('stores the session token and shows who is signed in', async () => {
		fakeApi();
		openAt('/sign-in');

		fireEvent.input(screen.getByLabelText(t('signIn.email')), { target: { value: 'a@b.c' } });
		fireEvent.input(screen.getByLabelText(t('signIn.password')), { target: { value: PASSWORD } });
		fireEvent.click(screen.getByRole('button', { name: t('signIn.submit') }));

		expect(
			await screen.findByText(t('home.signedInAs', { email: 'owner@example.com' }))
		).toBeTruthy();
		expect(token()).toBe(VALID_TOKEN);
	});

	it('shows an error and stores nothing when the credentials are wrong', async () => {
		fakeApi();
		openAt('/sign-in');

		fireEvent.input(screen.getByLabelText(t('signIn.email')), { target: { value: 'a@b.c' } });
		fireEvent.input(screen.getByLabelText(t('signIn.password')), { target: { value: 'wrong' } });
		fireEvent.click(screen.getByRole('button', { name: t('signIn.submit') }));

		expect((await screen.findByRole('alert')).textContent).toBe(t('signIn.invalid'));
		expect(token()).toBeNull();
		expect(sessionStorage.length).toBe(0);
	});
});

describe('an existing session', () => {
	it('returns to sign-in when the server no longer recognizes the session', async () => {
		// The session was revoked or expired: the user must not be left on a page that fails.
		fakeApi({ revokeOnMe: true });
		saveToken('stale-token');
		openAt('/');

		expect(await screen.findByRole('heading', { name: t('signIn.title') })).toBeTruthy();
		expect(token()).toBeNull();
		expect(sessionStorage.length).toBe(0);
	});

	it('sign out revokes the token on the server and forgets it locally', async () => {
		const calls = fakeApi();
		saveToken(VALID_TOKEN);
		openAt('/');
		await screen.findByText(t('home.signedInAs', { email: 'owner@example.com' }));

		fireEvent.click(screen.getByRole('button', { name: t('home.signOut') }));

		expect(await screen.findByRole('heading', { name: t('signIn.title') })).toBeTruthy();
		const logout = calls.find((call) => call.path === '/auth/logout');
		expect(logout?.method).toBe('POST');
		expect(logout?.authorization).toBe(`Bearer ${VALID_TOKEN}`);
		expect(token()).toBeNull();
	});
});
