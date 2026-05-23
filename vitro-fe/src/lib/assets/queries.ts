import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
	archiveAsset,
	createAsset,
	getAsset,
	listAssets,
	updateAsset,
	uploadAssetFile
} from '@/lib/assets/api';
import { useAuthStore } from '@/lib/state/auth';

export function assetsKey(workspaceId: string, datasetId: string) {
	return ['assets', workspaceId, datasetId] as const;
}

export function assetKey(workspaceId: string, assetId: string) {
	return ['asset', workspaceId, assetId] as const;
}

export function useAssetsQuery(datasetId: string) {
	const auth = useAuthStore((s) => s.auth)!;

	return useQuery({
		queryKey: assetsKey(auth.workspaceId, datasetId),
		queryFn: () =>
			listAssets({
				accessToken: auth.accessToken,
				workspaceId: auth.workspaceId,
				datasetId
			}),
		select: (res) => res.data
	});
}

export function useAssetQuery(assetId: string) {
	const auth = useAuthStore((s) => s.auth)!;

	return useQuery({
		queryKey: assetKey(auth.workspaceId, assetId),
		queryFn: () =>
			getAsset({
				accessToken: auth.accessToken,
				workspaceId: auth.workspaceId,
				assetId
			}),
		select: (res) => res.data
	});
}

export function useCreateAssetMutation() {
	const auth = useAuthStore((s) => s.auth)!;
	const qc = useQueryClient();

	return useMutation({
		mutationFn: (input: Parameters<typeof createAsset>[0]['input']) =>
			createAsset({ accessToken: auth.accessToken, workspaceId: auth.workspaceId, input }),
		onSuccess: (_res, input) =>
			qc.invalidateQueries({ queryKey: assetsKey(auth.workspaceId, input.dataset_id) })
	});
}

export function useUpdateAssetMutation() {
	const auth = useAuthStore((s) => s.auth)!;
	const qc = useQueryClient();

	return useMutation({
		mutationFn: (args: {
			assetId: string;
			datasetId: string;
			input: Parameters<typeof updateAsset>[0]['input'];
		}) =>
			updateAsset({
				accessToken: auth.accessToken,
				workspaceId: auth.workspaceId,
				assetId: args.assetId,
				input: args.input
			}),
		onSuccess: (_res, vars) => {
			qc.invalidateQueries({ queryKey: assetsKey(auth.workspaceId, vars.datasetId) });
			qc.invalidateQueries({ queryKey: assetKey(auth.workspaceId, vars.assetId) });
		}
	});
}

export function useArchiveAssetMutation() {
	const auth = useAuthStore((s) => s.auth)!;
	const qc = useQueryClient();

	return useMutation({
		mutationFn: (args: { assetId: string; datasetId: string }) =>
			archiveAsset({
				accessToken: auth.accessToken,
				workspaceId: auth.workspaceId,
				assetId: args.assetId
			}),
		onSuccess: (_res, vars) =>
			qc.invalidateQueries({ queryKey: assetsKey(auth.workspaceId, vars.datasetId) })
	});
}

export function useUploadAssetMutation() {
	const auth = useAuthStore((s) => s.auth)!;
	const qc = useQueryClient();

	return useMutation({
		mutationFn: (args: { assetId: string; file: File }) =>
			uploadAssetFile({
				accessToken: auth.accessToken,
				workspaceId: auth.workspaceId,
				assetId: args.assetId,
				file: args.file
			}),
		onSuccess: (_res, vars) =>
			qc.invalidateQueries({ queryKey: assetKey(auth.workspaceId, vars.assetId) })
	});
}
