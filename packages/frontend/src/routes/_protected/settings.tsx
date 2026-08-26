import { createFileRoute } from '@tanstack/react-router';
import { Eye, EyeOff, KeyRound, Save, Trash2 } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Separator } from '@/components/ui/separator';
import { DarkModeToggle } from '@/components/settings/dark-mode-toggle';
import { useEffect, useState } from 'react';

export const Route = createFileRoute('/_protected/settings')({
	head: () => ({
		meta: [{ title: 'Settings' }]
	}),
	component: RouteComponent,
	staticData: {
		breadcrumb: 'Settings'
	}
});

function getLocalStorageString(key: string, fallback = '') {
	if (typeof window === 'undefined') return fallback;
	return window.localStorage.getItem(key) ?? fallback;
}

function RouteComponent() {
	const [llmKey, setLlmKey] = useState<string>('');
	const [showKey, setShowKey] = useState(false);
	const [savedAt, setSavedAt] = useState<string | null>(null);

	useEffect(() => {
		setLlmKey(getLocalStorageString('llmApiKey', ''));
	}, []);

	function saveKey() {
		const trimmed = llmKey.trim();
		if (typeof window !== 'undefined') {
			window.localStorage.setItem('llmApiKey', trimmed);
		}
		setLlmKey(trimmed);
		setSavedAt(new Date().toLocaleString());
	}

	function clearKey() {
		if (typeof window !== 'undefined') {
			window.localStorage.removeItem('llmApiKey');
		}
		setLlmKey('');
		setSavedAt(new Date().toLocaleString());
	}

	return (
		<div className="flex flex-1 flex-col">
			<div className="@container/main flex flex-1 flex-col gap-6 px-4 py-4 md:px-6 md:py-6">
				<div className="flex flex-col gap-1">
					<h1 className="text-xl font-semibold tracking-tight">Settings</h1>
					<p className="text-muted-foreground text-sm">
						Manage appearance and API credentials for this workspace.
					</p>
				</div>

				<div className="grid gap-6 lg:grid-cols-4">
					<div className="space-y-6 lg:col-span-3">
						{/* Appearance */}
						<Card>
							<CardHeader>
								<CardTitle>Appearance</CardTitle>
								<CardDescription>Theme preference for this browser.</CardDescription>
							</CardHeader>
							<CardContent className="space-y-5">
								<div className="flex items-start justify-between gap-4">
									<div className="space-y-1">
										<Label htmlFor="dark-mode" className="font-medium">
											Dark mode
										</Label>
										<p className="text-muted-foreground text-sm">
											Toggle dark theme. Saved to local storage.
										</p>
									</div>

									<DarkModeToggle id="dark-mode" />
								</div>
							</CardContent>
						</Card>

						{/* LLM Key */}
						<Card>
							<CardHeader>
								<CardTitle className="flex items-center gap-2">
									<KeyRound className="h-4 w-4" />
									LLM API Key
								</CardTitle>
								<CardDescription>
									This example stores the key in localStorage. For production, store it server-side.
								</CardDescription>
							</CardHeader>

							<CardContent className="space-y-4">
								<div className="space-y-2">
									<Label htmlFor="llm-key">Key</Label>

									<div className="flex gap-2">
										<Input
											id="llm-key"
											type={showKey ? 'text' : 'password'}
											value={llmKey}
											onChange={(e) => setLlmKey(e.target.value)}
											placeholder="Paste your API key…"
											autoComplete="off"
											spellCheck={false}
										/>

										<Button
											type="button"
											variant="secondary"
											onClick={() => setShowKey((v) => !v)}
											aria-label={showKey ? 'Hide key' : 'Show key'}
										>
											{showKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
										</Button>
									</div>

									<p className="text-muted-foreground text-xs">
										Recommendation: proxy LLM calls through your backend and keep keys out of the
										browser.
									</p>
								</div>

								<Separator />

								<div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
									<div className="text-muted-foreground text-sm">
										{savedAt ? (
											<>Last updated: {savedAt}</>
										) : (
											<>No changes saved in this session.</>
										)}
									</div>

									<div className="flex gap-2">
										<Button type="button" onClick={saveKey}>
											<Save className="mr-2 h-4 w-4" />
											Save
										</Button>
										<Button type="button" variant="destructive" onClick={clearKey}>
											<Trash2 className="mr-2 h-4 w-4" />
											Clear
										</Button>
									</div>
								</div>
							</CardContent>
						</Card>
					</div>
				</div>
			</div>
		</div>
	);
}
