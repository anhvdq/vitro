import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow
} from '@/components/ui/table';
import { useAuthStore } from '@/lib/state/auth';
import { useTaskRunsQuery } from '@/lib/taskRuns/queries';
import { useTaskQuery } from '@/lib/tasks/queries';
import { createFileRoute, Link } from '@tanstack/react-router';
import { useEffect } from 'react';
import { toast } from 'sonner';

export const Route = createFileRoute('/_protected/tasks/$id')({
	head: () => ({
		meta: [{ title: 'Task' }]
	}),
	component: RouteComponent,
	staticData: {
		breadcrumb: 'Task'
	}
});

function formatDate(iso: string | null) {
	if (!iso) return '—';
	const d = new Date(iso);
	if (Number.isNaN(d.getTime())) return iso;
	return d.toLocaleString();
}

function RouteComponent() {
	const { id } = Route.useParams();
	const hydrated = useAuthStore((s) => s.hydrated);
	const auth = useAuthStore((s) => s.auth);
	if (!hydrated || !auth) return null;

	return <TaskDetailPage taskId={id} />;
}

function TaskDetailPage({ taskId }: { taskId: string }) {
	const taskQuery = useTaskQuery(taskId);
	const runsQuery = useTaskRunsQuery(taskId);

	useEffect(() => {
		if (!taskQuery.error) return;
		toast.error((taskQuery.error as any)?.message ?? 'Failed to load task');
	}, [taskQuery.error]);

	useEffect(() => {
		if (!runsQuery.error) return;
		toast.error((runsQuery.error as any)?.message ?? 'Failed to load runs');
	}, [runsQuery.error]);

	const task = taskQuery.data;
	if (!task) return null;

	const runs = runsQuery.data;
	if (!runs) return null;

	return (
		<div className="flex flex-col gap-6 py-4 md:py-6">
			<div className="px-4 lg:px-6">
				<div className="flex flex-wrap items-center justify-between gap-3">
					<div className="min-w-0">
						<div className="flex items-center gap-2">
							<h1 className="truncate text-xl font-semibold">Task</h1>
							<Badge variant="outline" className="text-muted-foreground">
								{task.status}
							</Badge>
						</div>
						<p className="text-muted-foreground mt-1 font-mono text-sm text-xs">{task.id}</p>
					</div>
				</div>

				<div className="mt-4 grid gap-2 text-sm">
					<div className="flex justify-between gap-4">
						<span className="text-muted-foreground">Task type</span>
						<span>{task.task_type}</span>
					</div>
					<div className="flex justify-between gap-4">
						<span className="text-muted-foreground">Asset</span>
						<Link
							className="font-mono text-xs underline"
							to="/assets/$id"
							params={{ id: task.asset_id }}
						>
							{task.asset_id}
						</Link>
					</div>
					<div className="flex justify-between gap-4">
						<span className="text-muted-foreground">Created</span>
						<span>{formatDate(task.created_at)}</span>
					</div>
				</div>
			</div>

			<div className="px-4 lg:px-6">
				<Separator className="mb-4" />
				<h2 className="text-sm font-medium">Runs</h2>
				<div className="mt-3 overflow-hidden rounded-lg border">
					<Table>
						<TableHeader className="bg-muted">
							<TableRow>
								<TableHead>Attempt</TableHead>
								<TableHead>Status</TableHead>
								<TableHead>Started</TableHead>
								<TableHead>Finished</TableHead>
								<TableHead>Error</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{runs.map((r) => (
								<TableRow key={r.id}>
									<TableCell className="font-mono text-xs">{r.attempt}</TableCell>
									<TableCell>
										<Badge variant="outline" className="text-muted-foreground">
											{r.status}
										</Badge>
									</TableCell>
									<TableCell className="text-muted-foreground">
										{formatDate(r.started_at)}
									</TableCell>
									<TableCell className="text-muted-foreground">
										{formatDate(r.finished_at)}
									</TableCell>
									<TableCell className="text-muted-foreground">{r.error_message ?? '—'}</TableCell>
								</TableRow>
							))}
							{runs.length === 0 ? (
								<TableRow>
									<TableCell colSpan={5} className="h-24 text-center text-sm">
										No runs.
									</TableCell>
								</TableRow>
							) : null}
						</TableBody>
					</Table>
				</div>
			</div>
		</div>
	);
}
