import { create } from 'zustand';
import { AuthState, UserSchema } from '@/lib/auth/types';

const ACCESS_TOKEN_KEY = 'vitro.access_token';
const WORKSPACE_ID_KEY = 'vitro.workspace_id';
const USER_KEY = 'vitro.user';

type AuthStore = {
	auth: AuthState | null;
	hydrated: boolean;
	hydrate: () => void;
	setAuth: (state: AuthState) => void;
	clearAuth: () => void;
};

function requireBrowser() {
	if (typeof window === 'undefined') {
		throw new Error('Browser environment required');
	}
}

export const useAuthStore = create<AuthStore>((set) => ({
	auth: null,
	hydrated: false,
	hydrate: () => {
		requireBrowser();
		const accessToken = window.localStorage.getItem(ACCESS_TOKEN_KEY);
		const workspaceId = window.localStorage.getItem(WORKSPACE_ID_KEY);
		const userRaw = window.localStorage.getItem(USER_KEY);

		if (!accessToken || !workspaceId || !userRaw) {
			set({ auth: null, hydrated: true });
			return;
		}

		const user = UserSchema.parse(JSON.parse(userRaw));
		set({ auth: { accessToken, workspaceId, user }, hydrated: true });
	},
	setAuth: (state) => {
		requireBrowser();
		window.localStorage.setItem(ACCESS_TOKEN_KEY, state.accessToken);
		window.localStorage.setItem(WORKSPACE_ID_KEY, state.workspaceId);
		window.localStorage.setItem(USER_KEY, JSON.stringify(state.user));
		set({ auth: state, hydrated: true });
	},
	clearAuth: () => {
		requireBrowser();
		window.localStorage.removeItem(ACCESS_TOKEN_KEY);
		window.localStorage.removeItem(WORKSPACE_ID_KEY);
		window.localStorage.removeItem(USER_KEY);
		set({ auth: null, hydrated: true });
	}
}));
