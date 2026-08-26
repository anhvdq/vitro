import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { createTask, getTask, listTasks } from '@/lib/tasks/api';
import { useAuthStore } from '@/lib/state/auth';

export function tasksKey(workspaceId: string) {
	return ['tasks', workspaceId] as const;
}

export function taskKey(workspaceId: string, taskId: string) {
	return ['task', workspaceId, taskId] as const;
}

export function useTasksQuery() {
	const auth = useAuthStore((s) => s.auth)!;

	return useQuery({
		queryKey: tasksKey(auth.workspaceId),
		queryFn: () => listTasks({ accessToken: auth.accessToken, workspaceId: auth.workspaceId }),
		select: (res) => res.data
	});
}

export function useTaskQuery(taskId: string) {
	const auth = useAuthStore((s) => s.auth)!;

	return useQuery({
		queryKey: taskKey(auth.workspaceId, taskId),
		queryFn: () =>
			getTask({ accessToken: auth.accessToken, workspaceId: auth.workspaceId, taskId }),
		select: (res) => res.data
	});
}

export function useCreateTaskMutation() {
	const auth = useAuthStore((s) => s.auth)!;
	const qc = useQueryClient();

	return useMutation({
		mutationFn: (input: Parameters<typeof createTask>[0]['input']) =>
			createTask({ accessToken: auth.accessToken, workspaceId: auth.workspaceId, input }),
		onSuccess: () => qc.invalidateQueries({ queryKey: tasksKey(auth.workspaceId) })
	});
}
