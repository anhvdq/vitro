import { useQuery } from '@tanstack/react-query';
import { listTaskRuns } from '@/lib/taskRuns/api';
import { useAuthStore } from '@/lib/state/auth';

export function taskRunsKey(workspaceId: string, taskId: string) {
	return ['taskRuns', workspaceId, taskId] as const;
}

export function useTaskRunsQuery(taskId: string) {
	const auth = useAuthStore((s) => s.auth)!;

	return useQuery({
		queryKey: taskRunsKey(auth.workspaceId, taskId),
		queryFn: () =>
			listTaskRuns({ accessToken: auth.accessToken, workspaceId: auth.workspaceId, taskId }),
		select: (res) => res.data
	});
}
