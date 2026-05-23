import z from 'zod';

export const TaskConfigsSchema = z
	.object({
		mode: z.string(),
		profiling: z
			.object({
				sampling: z.string(),
				sample_rows: z.number(),
				seed: z.number()
			})
			.optional(),
		generate: z.array(z.string()).optional()
	})
	.loose();

export const TaskSchema = z.object({
	id: z.string(),
	workspace_id: z.string(),
	dataset_id: z.string(),
	asset_id: z.string(),
	task_type: z.enum(['FULL_ANALYSIS']),
	status: z.enum(['QUEUED', 'RUNNING', 'SUCCEEDED', 'FAILED', 'CANCELED']),
	configs: TaskConfigsSchema,
	correlation_id: z.string(),
	created_at: z.string(),
	created_by: z.string(),
	created_by_name: z.string(),
	updated_at: z.string()
});

export type Task = z.infer<typeof TaskSchema>;
export type TaskConfigs = z.infer<typeof TaskConfigsSchema>;

export const TaskListResponseSchema = z.object({
	data: z.array(TaskSchema),
	page: z.number(),
	page_size: z.number(),
	total: z.number()
});

export type TaskListResponse = z.infer<typeof TaskListResponseSchema>;
