import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
	listDatasets,
	createDataset,
	updateDataset,
	archiveDataset,
	getDataset
} from '@/lib/datasets/api';
import { useAuthStore } from '@/lib/state/auth';

export function datasetsKey(workspaceId: string) {
	return ['datasets', workspaceId] as const;
}

export function datasetKey(workspaceId: string, datasetId: string) {
	return ['dataset', workspaceId, datasetId] as const;
}

export function useDatasetsQuery() {
	const auth = useAuthStore((s) => s.auth)!;

	return useQuery({
		queryKey: datasetsKey(auth.workspaceId),
		queryFn: () => listDatasets({ accessToken: auth.accessToken, workspaceId: auth.workspaceId }),
		select: (res) => res.data
	});
}

export function useDatasetQuery(datasetId: string) {
	const auth = useAuthStore((s) => s.auth)!;

	return useQuery({
		queryKey: datasetKey(auth.workspaceId, datasetId),
		queryFn: () =>
			getDataset({ accessToken: auth.accessToken, workspaceId: auth.workspaceId, datasetId }),
		select: (res) => res.data
	});
}

export function useCreateDatasetMutation() {
	const auth = useAuthStore((s) => s.auth)!;
	const qc = useQueryClient();

	return useMutation({
		mutationFn: (input: Parameters<typeof createDataset>[0]['input']) =>
			createDataset({ accessToken: auth.accessToken, workspaceId: auth.workspaceId, input }),
		onSuccess: () => qc.invalidateQueries({ queryKey: datasetsKey(auth.workspaceId) })
	});
}

export function useUpdateDatasetMutation() {
	const auth = useAuthStore((s) => s.auth)!;
	const qc = useQueryClient();

	return useMutation({
		mutationFn: (args: {
			datasetId: string;
			input: Parameters<typeof updateDataset>[0]['input'];
		}) =>
			updateDataset({
				accessToken: auth.accessToken,
				workspaceId: auth.workspaceId,
				datasetId: args.datasetId,
				input: args.input
			}),
		onSuccess: (_res, vars) => {
			qc.invalidateQueries({ queryKey: datasetsKey(auth.workspaceId) });
			qc.invalidateQueries({ queryKey: datasetKey(auth.workspaceId, vars.datasetId) });
		}
	});
}

export function useArchiveDatasetMutation() {
	const auth = useAuthStore((s) => s.auth)!;
	const qc = useQueryClient();

	return useMutation({
		mutationFn: (datasetId: string) =>
			archiveDataset({ accessToken: auth.accessToken, workspaceId: auth.workspaceId, datasetId }),
		onSuccess: () => qc.invalidateQueries({ queryKey: datasetsKey(auth.workspaceId) })
	});
}
