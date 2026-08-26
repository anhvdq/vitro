import { apiFetchJsonWorkspace } from '@/lib/api/client';
import { TaskRunGetResponseSchema, TaskRunListResponseSchema } from '@/lib/taskRuns/types';

export async function listTaskRuns(args: {
	accessToken: string;
	workspaceId: string;
	taskId: string;
	page?: number;
	pageSize?: number;
}) {
	return apiFetchJsonWorkspace({
		method: 'GET',
		path: `/tasks/${args.taskId}/runs`,
		query: { page: args.page, page_size: args.pageSize },
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: TaskRunListResponseSchema
	});
}

export async function getTaskRun(args: {
	accessToken: string;
	workspaceId: string;
	taskRunId: string;
}) {
	return apiFetchJsonWorkspace({
		method: 'GET',
		path: `/task-runs/${args.taskRunId}`,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: TaskRunGetResponseSchema
	});
}
