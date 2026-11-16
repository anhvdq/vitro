import { createQuery, useQueryClient } from '@tanstack/solid-query';
import { api } from '../../api/client';
import { clearToken, saveToken, token } from '../../api/session';

export type SignInFailure = 'invalid_credentials' | 'unavailable';

export class SignInError extends Error {
	constructor(readonly reason: SignInFailure) {
		super(reason);
	}
}

export async function signIn(email: string, password: string): Promise<void> {
	const result = await api.POST('/auth/login', { body: { email, password } }).catch(() => {
		throw new SignInError('unavailable');
	});
	if (result.response.status === 401) throw new SignInError('invalid_credentials');
	if (!result.data) throw new SignInError('unavailable');
	saveToken(result.data.data.access_token);
}

export function useCurrentUser() {
	return createQuery(() => ({
		queryKey: ['me'],
		enabled: token() !== null,
		retry: false,
		queryFn: async () => {
			const { data } = await api.GET('/auth/me');
			if (!data) throw new Error('could not load the current user');
			return data.data;
		}
	}));
}

export function useSignOut() {
	const queryClient = useQueryClient();
	return async () => {
		try {
			await api.POST('/auth/logout');
		} catch (error) {
			// The token is dropped locally either way; the server session then ends at expiry.
			console.error('sign-out request failed', error);
		}
		clearToken();
		queryClient.clear();
	};
}
