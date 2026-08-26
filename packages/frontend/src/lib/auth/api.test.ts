import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/lib/api/client', () => ({
	apiFetchJsonPublic: vi.fn()
}));

import { apiFetchJsonPublic } from '@/lib/api/client';
import { exchangeGoogleToken, loginWithPassword, signupWithPassword } from '@/lib/auth/api';

const apiFetchJsonPublicMock = vi.mocked(apiFetchJsonPublic);

describe('auth api', () => {
	beforeEach(() => {
		apiFetchJsonPublicMock.mockReset();
	});

	it('calls /auth/login and maps auth state', async () => {
		apiFetchJsonPublicMock.mockResolvedValue({
			data: {
				access_token: 'login-token',
				user: {
					id: 'u1',
					name: 'User',
					age: null,
					email: 'u@example.com',
					default_workspace_id: 'ws-1'
				}
			}
		});

		const result = await loginWithPassword({ email: 'u@example.com', password: 'secret' });

		expect(apiFetchJsonPublicMock).toHaveBeenCalledWith({
			method: 'POST',
			path: '/auth/login',
			body: { email: 'u@example.com', password: 'secret' },
			schema: expect.anything()
		});
		expect(result).toEqual({
			accessToken: 'login-token',
			workspaceId: 'ws-1',
			user: {
				id: 'u1',
				name: 'User',
				age: null,
				email: 'u@example.com',
				default_workspace_id: 'ws-1'
			}
		});
	});

	it('calls /auth/signup and maps auth state', async () => {
		apiFetchJsonPublicMock.mockResolvedValue({
			data: {
				access_token: 'signup-token',
				user: {
					id: 'u2',
					name: 'New User',
					age: null,
					email: 'new@example.com',
					default_workspace_id: 'ws-2'
				}
			}
		});

		const result = await signupWithPassword({
			name: 'New User',
			email: 'new@example.com',
			password: 'secret'
		});

		expect(apiFetchJsonPublicMock).toHaveBeenCalledWith({
			method: 'POST',
			path: '/auth/signup',
			body: { name: 'New User', email: 'new@example.com', password: 'secret' },
			schema: expect.anything()
		});
		expect(result.workspaceId).toBe('ws-2');
		expect(result.accessToken).toBe('signup-token');
	});

	it('calls /auth/exchange with google provider', async () => {
		apiFetchJsonPublicMock.mockResolvedValue({
			data: {
				access_token: 'exchange-token',
				user: {
					id: 'u3',
					name: 'Google User',
					age: null,
					email: 'google@example.com',
					default_workspace_id: 'ws-3'
				}
			}
		});

		await exchangeGoogleToken('google-token');

		expect(apiFetchJsonPublicMock).toHaveBeenCalledWith({
			method: 'POST',
			path: '/auth/exchange',
			body: { provider: 'google', token: 'google-token' },
			schema: expect.anything()
		});
	});
});
