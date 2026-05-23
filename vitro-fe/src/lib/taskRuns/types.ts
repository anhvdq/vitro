import z from 'zod';

export const TaskRunSchema = z.object({
	id: z.string(),
	task_id: z.string(),
	attempt: z.number(),
	status: z.enum(['RUNNING', 'SUCCEEDED', 'FAILED', 'CANCELED']),
	error_message: z.string().nullable(),
	started_at: z.string().nullable(),
	finished_at: z.string().nullable(),
	metrics: z.record(z.string(), z.union([z.string(), z.number(), z.boolean(), z.null()])),
	created_at: z.string()
});

export type TaskRun = z.infer<typeof TaskRunSchema>;

export const TaskRunListResponseSchema = z.object({
	data: z.array(TaskRunSchema),
	page: z.number(),
	page_size: z.number(),
	total: z.number()
});

export type TaskRunListResponse = z.infer<typeof TaskRunListResponseSchema>;

export const TaskRunGetResponseSchema = z.object({
	data: TaskRunSchema
});

export type TaskRunGetResponse = z.infer<typeof TaskRunGetResponseSchema>;
