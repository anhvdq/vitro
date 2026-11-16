import { createSignal } from 'solid-js';

// sessionStorage keeps the token for the tab only. This is an interim until the API sets an
// HttpOnly cookie; until then the token is readable by scripts on the page.
const KEY = 'vitro.session';

function read(): string | null {
	try {
		return sessionStorage.getItem(KEY);
	} catch {
		return null;
	}
}

const [token, setToken] = createSignal<string | null>(read());

export { token };

export function saveToken(value: string): void {
	try {
		sessionStorage.setItem(KEY, value);
	} catch {
		// Storage can be blocked; the session then lasts until the page reloads.
	}
	setToken(value);
}

export function clearToken(): void {
	try {
		sessionStorage.removeItem(KEY);
	} catch {
		// Nothing stored, nothing to clear.
	}
	setToken(null);
}
