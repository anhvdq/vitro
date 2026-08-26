import z from 'zod';
import { apiFetchJsonWorkspace } from '@/lib/api/client';
import { TaskListResponseSchema, TaskSchema, TaskConfigsSchema } from '@/lib/tasks/types';

const TaskGetResponseSchema = z.object({ data: TaskSchema });
const TaskCreateResponseSchema = z.object({ data: TaskSchema });

export type CreateTaskInput = {
	asset_id: string;
	task_type: 'FULL_ANALYSIS';
	configs: z.infer<typeof TaskConfigsSchema>;
};

export async function listTasks(args: {
	accessToken: string;
	workspaceId: string;
	page?: number;
	pageSize?: number;
	assetId?: string;
	status?: 'QUEUED' | 'RUNNING' | 'SUCCEEDED' | 'FAILED' | 'CANCELED';
	taskType?: 'FULL_ANALYSIS';
}) {
	return apiFetchJsonWorkspace({
		method: 'GET',
		path: '/tasks',
		query: {
			page: args.page,
			page_size: args.pageSize,
			asset_id: args.assetId,
			status: args.status,
			task_type: args.taskType
		},
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: TaskListResponseSchema
	});
}

export async function getTask(args: { accessToken: string; workspaceId: string; taskId: string }) {
	return apiFetchJsonWorkspace({
		method: 'GET',
		path: `/tasks/${args.taskId}`,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: TaskGetResponseSchema
	});
}

export async function createTask(args: {
	accessToken: string;
	workspaceId: string;
	input: CreateTaskInput;
}) {
	return apiFetchJsonWorkspace({
		method: 'POST',
		path: '/tasks',
		body: args.input,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: TaskCreateResponseSchema
	});
}
