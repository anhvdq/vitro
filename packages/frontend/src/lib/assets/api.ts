import z from 'zod';
import { apiFetchJsonWorkspace } from '@/lib/api/client';
import { AssetListResponseSchema, AssetSchema } from '@/lib/assets/types';

const AssetGetResponseSchema = z.object({ data: AssetSchema });
const AssetCreateResponseSchema = z.object({ data: AssetSchema });
const AssetUpdateResponseSchema = z.object({ data: AssetSchema });
const AssetArchiveResponseSchema = z.object({ data: AssetSchema });

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
	datasetId?: string;
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
