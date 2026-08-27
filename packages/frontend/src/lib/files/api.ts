import z from 'zod';
import { apiFetchFormWorkspace, apiFetchJsonWorkspace, throwIfError } from '@/lib/api/client';
import { env } from '@/lib/env';
import { FileListResponseSchema, FileResponseDTOSchema } from '@/lib/files/types';

const FileUploadResponseSchema = z.object({ data: FileResponseDTOSchema });

export async function listAssetFiles(args: {
	accessToken: string;
	workspaceId: string;
	assetId: string;
}) {
	return apiFetchJsonWorkspace({
		method: 'GET',
		path: `/assets/${args.assetId}/files`,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: FileListResponseSchema
	});
}

export async function uploadAssetFile(args: {
	accessToken: string;
	workspaceId: string;
	assetId: string;
	file: File;
}) {
	const form = new FormData();
	form.set('file', args.file);

	return apiFetchFormWorkspace({
		method: 'POST',
		path: `/assets/${args.assetId}/upload`,
		body: form,
		accessToken: args.accessToken,
		workspaceId: args.workspaceId,
		schema: FileUploadResponseSchema
	});
}

export async function downloadAssetFile(args: {
	accessToken: string;
	workspaceId: string;
	assetId: string;
	fileId: string;
	fileName: string;
}) {
	const base = env.apiBaseUrl.endsWith('/') ? env.apiBaseUrl.slice(0, -1) : env.apiBaseUrl;
	const url = `${base}/assets/${args.assetId}/files/${args.fileId}/download`;

	const res = await fetch(url, {
		method: 'GET',
		headers: {
			Authorization: `Bearer ${args.accessToken}`,
			'X-Workspace-Id': args.workspaceId
		}
	});

	if (!res.ok) {
		await throwIfError(res);
	}

	const blob = await res.blob();
	const objectUrl = URL.createObjectURL(blob);
	const link = document.createElement('a');
	link.href = objectUrl;
	link.download = args.fileName;
	document.body.appendChild(link);
	link.click();
	document.body.removeChild(link);
	URL.revokeObjectURL(objectUrl);
}
