import { createFileRoute, Outlet } from '@tanstack/react-router';

export const Route = createFileRoute('/_protected/assets')({
	head: () => ({
		meta: [{ title: 'Assets' }]
	}),
	component: RouteComponent,
	staticData: {
		breadcrumb: 'Assets'
	}
});

function RouteComponent() {
	return (
		<div className="flex flex-1 flex-col">
			<div className="@container/main flex flex-1 flex-col gap-2">
				<Outlet />
			</div>
		</div>
	);
}
