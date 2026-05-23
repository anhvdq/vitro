import { ReactNode, useEffect, useState } from 'react';
import { HeadContent, Outlet, Scripts, createRootRoute } from '@tanstack/react-router';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

import appCssUrl from '../app.css?url';
import { useAuthStore } from '@/lib/state/auth';

const darkModeScript = `
let darkModeMediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

updateMode();
darkModeMediaQuery.addEventListener('change', updateModeWithoutTransitions);
window.addEventListener('storage', updateModeWithoutTransitions);

function updateMode() {
	let isSystemDarkMode = darkModeMediaQuery.matches;
	let isDarkMode =
		window.localStorage.isDarkMode === 'true' ||
		(!('isDarkMode' in window.localStorage) && isSystemDarkMode);

	if (isDarkMode) {
		document.documentElement.classList.add('dark');
	} else {
		document.documentElement.classList.remove('dark');
	}

	if (isDarkMode === isSystemDarkMode) {
		delete window.localStorage.isDarkMode;
	}
}

function disableTransitionsTemporarily() {
	document.documentElement.classList.add('**:transition-none!');
	window.setTimeout(() => {
		document.documentElement.classList.remove('**:transition-none!');
	}, 0);
}

function updateModeWithoutTransitions() {
	disableTransitionsTemporarily();
	updateMode();
}
`;

export const Route = createRootRoute({
	component: RootComponent
});

function RootComponent() {
	const [queryClient] = useState(
		() =>
			new QueryClient({
				defaultOptions: {
					queries: { retry: false, refetchOnWindowFocus: false },
					mutations: { retry: false }
				}
			})
	);

	useEffect(() => {
		useAuthStore.getState().hydrate();
	}, []);

	return (
		<RootDocument>
			<QueryClientProvider client={queryClient}>
				<Outlet />
			</QueryClientProvider>
		</RootDocument>
	);
}

function RootDocument({ children }: Readonly<{ children: ReactNode }>) {
	return (
		<html lang="en" suppressHydrationWarning>
			<head>
				<meta charSet="utf-8" />
				<meta name="viewport" content="width=device-width, initial-scale=1" />
				<link rel="icon" href="/favicon.ico" />
				<link rel="stylesheet" href={appCssUrl} />
				<HeadContent />
				<script dangerouslySetInnerHTML={{ __html: darkModeScript }} />
			</head>
			<body className="bg-white text-zinc-600 dark:bg-zinc-900 dark:text-zinc-400">
				{children}
				<Scripts />
			</body>
		</html>
	);
}
