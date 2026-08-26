import z from 'zod';

export const DatasetSchema = z.object({
	id: z.string(),
	workspace_id: z.string(),
	name: z.string(),
	display_name: z.string(),
	description: z.string().nullable(),
	owner_user_id: z.string(),
	owner_user_name: z.string(),
	tags: z.array(z.string()),
	status: z.enum(['ACTIVE', 'ARCHIVED']),
	created_at: z.string(),
	updated_at: z.string(),
	archived_at: z.string().nullable()
});

export type Dataset = z.infer<typeof DatasetSchema>;

export const DatasetListResponseSchema = z.object({
	data: z.array(DatasetSchema),
	page: z.number(),
	page_size: z.number(),
	total: z.number()
});

export type DatasetListResponse = z.infer<typeof DatasetListResponseSchema>;
