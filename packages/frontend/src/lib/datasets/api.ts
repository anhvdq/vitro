import z from 'zod';
import { apiFetchJsonWorkspace } from '@/lib/api/client';
import { DatasetListResponseSchema, DatasetSchema } from '@/lib/datasets/types';

const DatasetGetResponseSchema = z.object({ data: DatasetSchema });
const DatasetCreateResponseSchema = z.object({ data: DatasetSchema });
const DatasetUpdateResponseSchema = z.object({ data: DatasetSchema });
const DatasetArchiveResponseSchema = z.object({ data: DatasetSchema });

export type CreateDatasetInput = {
	name: string;
	display_name: string;
	description: string;
	tags: string[];
};

export type UpdateDatasetInput = {
	display_name: string;
	description: string;
	owner_user_id: string;
	tags: string[];
};

export async function listDatasets(args: {
	accessToken: string;
	workspaceId: string;
	page?: number;
	pageSize?: number;
	status?: 'ACTIVE' | 'ARCHIVED';
	search?: string;
	tags?: string[];
}) {
	return apiFetchJsonWorkspace({
		method: 'GET',
		path: '/datasets',
		query: {
			page: args.page,
			page_size: args.pageSize,
			status: args.status,
			search: args.search,
			tags: args.tags ? args.tags.join(',') : undefined
		},
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: DatasetListResponseSchema
	});
}

export async function getDataset(args: {
	accessToken: string;
	workspaceId: string;
	datasetId: string;
}) {
	return apiFetchJsonWorkspace({
		method: 'GET',
		path: `/datasets/${args.datasetId}`,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: DatasetGetResponseSchema
	});
}

export async function createDataset(args: {
	accessToken: string;
	workspaceId: string;
	input: CreateDatasetInput;
}) {
	return apiFetchJsonWorkspace({
		method: 'POST',
		path: '/datasets',
		body: args.input,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: DatasetCreateResponseSchema
	});
}

export async function updateDataset(args: {
	accessToken: string;
	workspaceId: string;
	datasetId: string;
	input: Partial<UpdateDatasetInput>;
}) {
	return apiFetchJsonWorkspace({
		method: 'PATCH',
		path: `/datasets/${args.datasetId}`,
		body: args.input,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: DatasetUpdateResponseSchema
	});
}

export async function archiveDataset(args: {
	accessToken: string;
	workspaceId: string;
	datasetId: string;
}) {
	return apiFetchJsonWorkspace({
		method: 'DELETE',
		path: `/datasets/${args.datasetId}`,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: DatasetArchiveResponseSchema
	});
}
