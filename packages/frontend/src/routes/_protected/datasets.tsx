import { createFileRoute, Outlet } from '@tanstack/react-router';

export const Route = createFileRoute('/_protected/datasets')({
	head: () => ({
		meta: [{ title: 'Datasets' }]
	}),
	component: RouteComponent,
	staticData: {
		breadcrumb: 'Datasets'
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
