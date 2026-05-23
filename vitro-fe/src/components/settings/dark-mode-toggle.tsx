'use client';

import { useEffect, useState } from 'react';

import { Switch } from '@/components/ui/switch';

function disableTransitionsTemporarily() {
	document.documentElement.classList.add('**:transition-none!');
	window.setTimeout(() => {
		document.documentElement.classList.remove('**:transition-none!');
	}, 0);
}

export function DarkModeToggle({ id }: { id: string }) {
	const [isDarkMode, setIsDarkMode] = useState(false);

	useEffect(() => {
		setIsDarkMode(document.documentElement.classList.contains('dark'));
	}, []);

	return (
		<Switch
			id={id}
			checked={isDarkMode}
			onCheckedChange={(next) => {
				setIsDarkMode(next);
				if (typeof window === 'undefined') return;

				disableTransitionsTemporarily();

				if (next) {
					document.documentElement.classList.add('dark');
				} else {
					document.documentElement.classList.remove('dark');
				}

				const isSystemDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
				if (next === isSystemDarkMode) {
					window.localStorage.removeItem('isDarkMode');
				} else {
					window.localStorage.setItem('isDarkMode', String(next));
				}
			}}
			aria-label="Toggle dark mode"
		/>
	);
}
