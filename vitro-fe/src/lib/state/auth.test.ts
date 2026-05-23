import { describe, expect, it } from 'vitest';
import { useAuthStore } from '@/lib/state/auth';

describe('useAuthStore', () => {
	it('hydrates from localStorage', () => {
		window.localStorage.setItem('vitro.access_token', 'token');
		window.localStorage.setItem('vitro.workspace_id', 'ws');
		window.localStorage.setItem(
			'vitro.user',
			JSON.stringify({
				id: 'u1',
				name: 'User',
				age: null,
				email: 'u@example.com',
				default_workspace_id: 'ws'
			})
		);

		useAuthStore.getState().hydrate();
		expect(useAuthStore.getState().auth).toEqual({
			accessToken: 'token',
			workspaceId: 'ws',
			user: {
				id: 'u1',
				name: 'User',
				age: null,
				email: 'u@example.com',
				default_workspace_id: 'ws'
			}
		});
	});
});
