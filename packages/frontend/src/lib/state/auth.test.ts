import { beforeEach, describe, expect, it } from 'vitest';
import { useAuthStore } from '@/lib/state/auth';

describe('useAuthStore', () => {
	beforeEach(() => {
		const values = new Map<string, string>();
		const storage: Storage = {
			get length() {
				return values.size;
			},
			clear: () => values.clear(),
			getItem: (key) => values.get(key) ?? null,
			key: (index) => [...values.keys()][index] ?? null,
			removeItem: (key) => values.delete(key),
			setItem: (key, value) => values.set(key, value)
		};

		Object.defineProperty(window, 'localStorage', {
			configurable: true,
			value: storage
		});
		useAuthStore.setState({ auth: null, hydrated: false });
	});

	it('restores a returning user session from browser storage', () => {
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
