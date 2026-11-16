import tailwindcss from '@tailwindcss/vite';
import solid from 'vite-plugin-solid';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [tailwindcss(), solid()],
	server: { port: 5174 },
	resolve: { conditions: ['development', 'browser'] },
	test: {
		environment: 'jsdom',
		setupFiles: ['./src/test-setup.ts'],
		environmentOptions: { jsdom: { url: 'http://localhost/' } },
		env: { VITE_API_BASE_URL: 'http://api.test' },
		server: { deps: { inline: [/solid-js/, /@solidjs/, /@tanstack\/solid-query/] } }
	}
});
