import { createFileRoute, Link, Outlet, useMatches } from '@tanstack/react-router';
import { LeftSidebar } from '@/components/left-sidebar';
import {
	Breadcrumb,
	BreadcrumbItem,
	BreadcrumbLink,
	BreadcrumbList,
	BreadcrumbPage,
	BreadcrumbSeparator
} from '@/components/ui/breadcrumb';
import { Separator } from '@/components/ui/separator';
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar';
import { platformName } from '@/lib/info';
import { Fragment } from 'react/jsx-runtime';
import { Toaster } from '@/components/ui/sonner';
import { RouteStaticData } from '@/lib/types';
import { useEffect } from 'react';
import { useRouter } from '@tanstack/react-router';
import { useAuthStore } from '@/lib/state/auth';

export const Route = createFileRoute('/_protected')({
	head: () => ({
		meta: [{ title: platformName }]
	}),
	component: RouteComponent,
	staticData: {
		breadcrumb: '#'
	}
});

function RouteComponent() {
	const router = useRouter();
	const hydrated = useAuthStore((s) => s.hydrated);
	const auth = useAuthStore((s) => s.auth);

	useEffect(() => {
		if (hydrated && !auth) {
			router.navigate({ to: '/login' });
		}
	}, [hydrated, auth, router]);

	if (!hydrated || !auth) return null;

	return (
		<SidebarProvider>
			<LeftSidebar />
			<SidebarInset>
				<header className="flex h-16 shrink-0 items-center gap-2">
					<div className="flex items-center gap-2 px-4">
						<SidebarTrigger className="-ml-1" />
						<Separator orientation="vertical" className="mr-2 data-[orientation=vertical]:h-4" />
						<AutoBreadcrumbs />
					</div>
				</header>
				<Outlet />
			</SidebarInset>
			<Toaster />
		</SidebarProvider>
	);
}

function AutoBreadcrumbs() {
	const matches = useMatches();

	const crumbs = matches
		.map((m) => {
			const breadcrumb = m.routeId && (m.staticData as RouteStaticData).breadcrumb;
			return { match: m, breadcrumb };
		})
		.filter((x) => x.breadcrumb);

	const getLabel = (bc: any, match: any) => (typeof bc === 'function' ? bc(match) : String(bc));

	return (
		<Breadcrumb>
			<BreadcrumbList>
				{crumbs.map(({ match, breadcrumb }, idx) => {
					const label = getLabel(breadcrumb, match);
					const isLast = idx === crumbs.length - 1;

					return (
						<Fragment key={match.id ?? match.routeId}>
							{idx > 0 ? <BreadcrumbSeparator className="hidden md:block" /> : null}
							<BreadcrumbItem key={match.id ?? match.routeId}>
								{isLast ? (
									<BreadcrumbPage>{label}</BreadcrumbPage>
								) : (
									// Use TanStack Link so it preserves SPA navigation
									<BreadcrumbLink asChild>
										<Link to={match.pathname}>{label}</Link>
									</BreadcrumbLink>
								)}
							</BreadcrumbItem>
						</Fragment>
					);
				})}
			</BreadcrumbList>
		</Breadcrumb>
	);
}
