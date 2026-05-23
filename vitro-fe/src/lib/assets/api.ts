import z from 'zod';
import { apiFetchJsonWorkspace } from '@/lib/api/client';
import { env } from '@/lib/env';
import { ApiErrorBodySchema, VitroApiError } from '@/lib/api/errors';
import { AssetListResponseSchema, AssetSchema } from '@/lib/assets/types';

const AssetGetResponseSchema = z.object({ data: AssetSchema });
const AssetCreateResponseSchema = z.object({ data: AssetSchema });
const AssetUpdateResponseSchema = z.object({ data: AssetSchema });
const AssetArchiveResponseSchema = z.object({ data: AssetSchema });

const AssetUploadResponseSchema = z.object({
	data: z.object({
		path: z.string(),
		file_name: z.string(),
		size_bytes: z.number(),
		content_type: z.string()
	})
});

export type CreateAssetInput = {
	dataset_id: string;
	name: string;
	display_name: string;
	description: string;
	tags: string[];
};

export type UpdateAssetInput = {
	display_name: string;
	description: string;
	owner_user_id: string;
	tags: string[];
};

export async function listAssets(args: {
	accessToken: string;
	workspaceId: string;
	datasetId: string;
	page?: number;
	pageSize?: number;
	status?: 'ACTIVE' | 'ARCHIVED';
	search?: string;
	tags?: string[];
}) {
	return apiFetchJsonWorkspace({
		method: 'GET',
		path: '/assets',
		query: {
			page: args.page,
			page_size: args.pageSize,
			dataset_id: args.datasetId,
			status: args.status,
			search: args.search,
			tags: args.tags ? args.tags.join(',') : undefined
		},
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: AssetListResponseSchema
	});
}

export async function getAsset(args: {
	accessToken: string;
	workspaceId: string;
	assetId: string;
}) {
	return apiFetchJsonWorkspace({
		method: 'GET',
		path: `/assets/${args.assetId}`,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: AssetGetResponseSchema
	});
}

export async function createAsset(args: {
	accessToken: string;
	workspaceId: string;
	input: CreateAssetInput;
}) {
	return apiFetchJsonWorkspace({
		method: 'POST',
		path: '/assets',
		body: args.input,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: AssetCreateResponseSchema
	});
}

export async function updateAsset(args: {
	accessToken: string;
	workspaceId: string;
	assetId: string;
	input: Partial<UpdateAssetInput>;
}) {
	return apiFetchJsonWorkspace({
		method: 'PATCH',
		path: `/assets/${args.assetId}`,
		body: args.input,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: AssetUpdateResponseSchema
	});
}

export async function archiveAsset(args: {
	accessToken: string;
	workspaceId: string;
	assetId: string;
}) {
	return apiFetchJsonWorkspace({
		method: 'DELETE',
		path: `/assets/${args.assetId}`,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: AssetArchiveResponseSchema
	});
}

export async function uploadAssetFile(args: {
	accessToken: string;
	workspaceId: string;
	assetId: string;
	file: File;
}) {
	const base = env.apiBaseUrl.endsWith('/') ? env.apiBaseUrl.slice(0, -1) : env.apiBaseUrl;
	const url = `${base}/assets/${args.assetId}/upload`;

	const form = new FormData();
	form.set('file', args.file);

	const res = await fetch(url, {
		method: 'POST',
		headers: {
			Authorization: `Bearer ${args.accessToken}`,
			'X-Workspace-Id': args.workspaceId
		},
		body: form
	});

	const json = await res.json();
	if (!res.ok) {
		const parsed = ApiErrorBodySchema.parse(json);
		throw new VitroApiError({
			status: res.status,
			code: parsed.error.code,
			message: parsed.error.message,
			correlationId: parsed.error.correlation_id,
			details: parsed.error.details
		});
	}

	return AssetUploadResponseSchema.parse(json);
}
