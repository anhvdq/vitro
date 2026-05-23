import { apiFetchJsonPublic } from '@/lib/api/client';
import { AuthSessionResponseSchema } from '@/lib/auth/types';

export async function loginWithPassword(input: { email: string; password: string }) {
	const res = await apiFetchJsonPublic({
		method: 'POST',
		path: '/auth/login',
		body: input,
		schema: AuthSessionResponseSchema
	});

	return {
		accessToken: res.data.access_token,
		workspaceId: res.data.user.default_workspace_id,
		user: res.data.user
	};
}

export async function signupWithPassword(input: { email: string; password: string; name: string }) {
	const res = await apiFetchJsonPublic({
		method: 'POST',
		path: '/auth/signup',
		body: input,
		schema: AuthSessionResponseSchema
	});

	return {
		accessToken: res.data.access_token,
		workspaceId: res.data.user.default_workspace_id,
		user: res.data.user
	};
}

export async function exchangeGoogleToken(token: string) {
	const res = await apiFetchJsonPublic({
		method: 'POST',
		path: '/auth/exchange',
		body: { provider: 'google', token },
		schema: AuthSessionResponseSchema
	});

	return {
		accessToken: res.data.access_token,
		workspaceId: res.data.user.default_workspace_id,
		user: res.data.user
	};
}
