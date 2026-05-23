import { createRouter, useRouter } from '@tanstack/react-router';
import { routeTree } from './routeTree.gen';
import { Button } from '@/components/ui/button';
import { ChevronLeft, Home } from 'lucide-react';

function NotFound() {
	const router = useRouter();

	return (
		<main className="min-h-[70vh] w-full px-6 py-16">
			<div className="mx-auto max-w-xl text-center">
				<h1 className="mt-6 text-2xl font-semibold tracking-tight text-zinc-900 sm:text-3xl dark:text-zinc-50">
					Page not found
				</h1>
				<p className="mt-3 text-sm leading-6 text-zinc-600 sm:text-base dark:text-zinc-200">
					The page you’re looking for doesn’t exist or may have been moved.
				</p>

				<div className="mt-8 flex flex-col items-center justify-center gap-6 sm:flex-row">
					<Button
						className="py-5 text-sm"
						variant="default"
						size="sm"
						onClick={() => router.history.back()}
					>
						<ChevronLeft />
						<span className="hidden lg:inline">Go Back</span>
					</Button>

					<Button
						className="py-5 text-sm"
						variant="default"
						size="sm"
						onClick={() => router.navigate({ to: '/' })}
					>
						<Home />
						<span className="hidden lg:inline">Go to home</span>
					</Button>
				</div>
			</div>
		</main>
	);
}

export function getRouter() {
	return createRouter({
		routeTree,
		scrollRestoration: true,
		defaultNotFoundComponent: NotFound,
		notFoundMode: 'root'
	});
}

declare module '@tanstack/react-router' {
	interface Register {
		router: ReturnType<typeof getRouter>;
	}
}
