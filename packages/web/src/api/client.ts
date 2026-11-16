import createClient, { type Middleware } from 'openapi-fetch';
import { clearToken, token } from './session';
import type { paths } from './schema';

const baseUrl = import.meta.env.VITE_API_BASE_URL;
if (!baseUrl) {
	throw new Error('VITE_API_BASE_URL is not set; copy .env.example to .env');
}

const session: Middleware = {
	onRequest({ request }) {
		const current = token();
		if (current) request.headers.set('Authorization', `Bearer ${current}`);
		return request;
	},
	onResponse({ response }) {
		// The server no longer knows this session (expired or revoked): forget it so the
		// app returns to sign-in instead of showing pages that fail silently.
		if (response.status === 401 && token()) clearToken();
		return undefined;
	}
};

export const api = createClient<paths>({
	baseUrl,
	// Resolved per call so tests can replace fetch.
	fetch: (request) => globalThis.fetch(request)
});
api.use(session);
