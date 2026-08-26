import z from 'zod';

export const AssetConnectorSchema = z.object({
	id: z.string(),
	type: z.string(),
	storage_prefix: z.string(),
	allowed_formats: z.array(z.string())
});

export const AssetSchema = z.object({
	id: z.string(),
	workspace_id: z.string(),
	dataset_id: z.string(),
	name: z.string(),
	display_name: z.string(),
	description: z.string().nullable(),
	owner_user_id: z.string(),
	owner_user_name: z.string(),
	tags: z.array(z.string()),
	connector: AssetConnectorSchema,
	status: z.enum(['ACTIVE', 'ARCHIVED']),
	created_at: z.string(),
	updated_at: z.string(),
	archived_at: z.string().nullable()
});

export type Asset = z.infer<typeof AssetSchema>;

export const AssetListResponseSchema = z.object({
	data: z.array(AssetSchema),
	page: z.number(),
	page_size: z.number(),
	total: z.number()
});

export type AssetListResponse = z.infer<typeof AssetListResponseSchema>;
