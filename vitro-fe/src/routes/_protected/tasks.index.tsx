import { TaskTable } from '@/components/tasks/task-table';
import { TaskUpsertInput, TaskUpsertSheet } from '@/components/tasks/form';
import { useCreateTaskMutation, useTasksQuery } from '@/lib/tasks/queries';
import { useAuthStore } from '@/lib/state/auth';
import { createFileRoute, useRouter } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

export const Route = createFileRoute('/_protected/tasks/')({
	head: () => ({
		meta: [{ title: 'Tasks' }]
	}),
	component: RouteComponent,
	staticData: {
		breadcrumb: 'Tasks'
	}
});

function RouteComponent() {
	const hydrated = useAuthStore((s) => s.hydrated);
	const auth = useAuthStore((s) => s.auth);
	if (!hydrated || !auth) return null;

	return <TasksPage />;
}

function TasksPage() {
	const router = useRouter();
	const tasksQuery = useTasksQuery();
	const createMutation = useCreateTaskMutation();
	const [open, setOpen] = useState(false);

	useEffect(() => {
		if (!tasksQuery.error) return;
		toast.error((tasksQuery.error as any)?.message ?? 'Failed to load tasks');
	}, [tasksQuery.error]);

	async function handleSubmit(payload: TaskUpsertInput) {
		await createMutation.mutateAsync(payload);
	}

	const tasks = tasksQuery.data;
	if (!tasks) return null;

	return (
		<div className="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
			<TaskTable
				data={tasks}
				onCreateTask={() => setOpen(true)}
				onViewTask={(task) => router.navigate({ to: '/tasks/$id', params: { id: task.id } })}
			/>
			<TaskUpsertSheet
				open={open}
				onOpenChange={setOpen}
				mode="create"
				task={null}
				onSubmit={handleSubmit}
			/>
		</div>
	);
}
