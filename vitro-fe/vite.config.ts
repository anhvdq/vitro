import { fileURLToPath, URL } from 'node:url';
import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { tanstackStart } from '@tanstack/react-start/plugin/vite';
import { defineConfig } from 'vite';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

export default defineConfig({
	plugins: [
		tailwindcss(),
		tanstackStart({
			prerender: {
				enabled: false
			},
			sitemap: {
				host: 'anhvdq.github.io'
			}
		}),
		nodePolyfills({
			// To make Buffer global
			exclude: ['stream'],
			globals: {
				Buffer: true
			}
		}),

		react()
	],
	publicDir: 'static',
	resolve: {
		alias: {
			'@': fileURLToPath(new URL('./src', import.meta.url))
		}
	},
	server: {
		fs: {
			allow: ['./']
		}
	}
});
