import z from 'zod';

export const FileResponseDTOSchema = z.object({
	id: z.string(),
	asset_id: z.string(),
	workspace_id: z.string(),
	file_name: z.string(),
	size_bytes: z.number(),
	content_type: z.string().nullable(),
	uploaded_by: z.string().nullable(),
	created_at: z.string()
});

export type FileResponseDTO = z.infer<typeof FileResponseDTOSchema>;

export const FileListResponseSchema = z.object({
	data: z.array(FileResponseDTOSchema)
});

export type FileListResponse = z.infer<typeof FileListResponseSchema>;
