import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { listAssetFiles, uploadAssetFile } from '@/lib/files/api';
import { useAuthStore } from '@/lib/state/auth';

export function assetFilesKey(workspaceId: string, assetId: string) {
	return ['asset-files', workspaceId, assetId] as const;
}

export function useAssetFilesQuery(assetId: string) {
	const auth = useAuthStore((s) => s.auth)!;

	return useQuery({
		queryKey: assetFilesKey(auth.workspaceId, assetId),
		queryFn: () =>
			listAssetFiles({
				accessToken: auth.accessToken,
				workspaceId: auth.workspaceId,
				assetId
			}),
		select: (res) => res.data
	});
}

export type UploadAssetFileResult = { file: File; status: 'success' | 'error'; error?: unknown };

export function useUploadAssetFilesMutation() {
	const auth = useAuthStore((s) => s.auth)!;
	const qc = useQueryClient();

	return useMutation({
		mutationFn: async (args: { assetId: string; files: File[] }) => {
			const settled = await Promise.allSettled(
				args.files.map((file) =>
					uploadAssetFile({
						accessToken: auth.accessToken,
						workspaceId: auth.workspaceId,
						assetId: args.assetId,
						file
					})
				)
			);

			const results: UploadAssetFileResult[] = settled.map((outcome, index) => {
				const file = args.files[index];
				if (outcome.status === 'fulfilled') return { file, status: 'success' };
				return { file, status: 'error', error: outcome.reason };
			});

			return results;
		},
		onSuccess: (_results, vars) =>
			qc.invalidateQueries({ queryKey: assetFilesKey(auth.workspaceId, vars.assetId) })
	});
}
