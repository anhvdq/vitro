import { AssetTable } from '@/components/assets/asset-table';
import { AssetUpsertInput, AssetUpsertSheet } from '@/components/assets/form';
import { Badge } from '@/components/ui/badge';
import { Asset } from '@/lib/assets/types';
import {
	useArchiveAssetMutation,
	useAssetsQuery,
	useCreateAssetMutation,
	useUpdateAssetMutation
} from '@/lib/assets/queries';
import { useDatasetQuery } from '@/lib/datasets/queries';
import { useAuthStore } from '@/lib/state/auth';
import { createFileRoute, useRouter } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

export const Route = createFileRoute('/_protected/datasets/$id')({
	head: () => ({
		meta: [{ title: 'Dataset' }]
	}),
	component: RouteComponent,
	staticData: {
		breadcrumb: 'Dataset'
	}
});

type SheetMode = 'create' | 'edit' | 'view';

function RouteComponent() {
	const { id } = Route.useParams();
	const hydrated = useAuthStore((s) => s.hydrated);
	const auth = useAuthStore((s) => s.auth);
	if (!hydrated || !auth) return null;

	return <DatasetDetailPage datasetId={id} />;
}

function DatasetDetailPage({ datasetId }: { datasetId: string }) {
	const router = useRouter();
	const datasetQuery = useDatasetQuery(datasetId);
	const assetsQuery = useAssetsQuery(datasetId);
	const createMutation = useCreateAssetMutation();
	const updateMutation = useUpdateAssetMutation();
	const archiveMutation = useArchiveAssetMutation();

	const [open, setOpen] = useState(false);
	const [mode, setMode] = useState<SheetMode>('create');
	const [active, setActive] = useState<Asset | null>(null);

	useEffect(() => {
		if (!datasetQuery.error) return;
		toast.error((datasetQuery.error as any)?.message ?? 'Failed to load dataset');
	}, [datasetQuery.error]);

	useEffect(() => {
		if (!assetsQuery.error) return;
		toast.error((assetsQuery.error as any)?.message ?? 'Failed to load assets');
	}, [assetsQuery.error]);

	async function handleSubmit(
		payload: AssetUpsertInput,
		ctx: { mode: SheetMode; assetId?: string }
	) {
		if (ctx.mode === 'create') {
			await createMutation.mutateAsync(payload);
			return;
		}

		if (!ctx.assetId) throw new Error('Missing asset id');

		await updateMutation.mutateAsync({
			assetId: ctx.assetId,
			datasetId,
			input: {
				display_name: payload.display_name,
				description: payload.description,
				tags: payload.tags
			}
		});
	}

	const dataset = datasetQuery.data;
	if (!dataset) return null;

	const assets = assetsQuery.data;
	if (!assets) return null;

	return (
		<div className="flex flex-col gap-6 py-4 md:py-6">
			<div className="px-4 lg:px-6">
				<div className="flex flex-wrap items-center justify-between gap-3">
					<div className="min-w-0">
						<div className="flex items-center gap-2">
							<h1 className="truncate text-xl font-semibold">{dataset.display_name}</h1>
							<Badge variant="outline" className="text-muted-foreground">
								{dataset.status}
							</Badge>
						</div>
						<p className="text-muted-foreground mt-1 text-sm">{dataset.description}</p>
					</div>
				</div>

				<div className="mt-4 grid gap-2 text-sm">
					<div className="flex justify-between gap-4">
						<span className="text-muted-foreground">Dataset ID</span>
						<span className="font-mono text-xs">{dataset.id}</span>
					</div>
					<div className="flex justify-between gap-4">
						<span className="text-muted-foreground">Owner</span>
						<span>{dataset.owner_user_name}</span>
					</div>
				</div>
			</div>

			<AssetTable
				data={assets}
				onCreateAsset={() => {
					setMode('create');
					setActive(null);
					setOpen(true);
				}}
				onEditAsset={(asset) => {
					setMode('edit');
					setActive(asset);
					setOpen(true);
				}}
				onViewAsset={(asset) => {
					router.navigate({ to: '/assets/$id', params: { id: asset.id } });
				}}
				onDeleteAsset={(assetId) => {
					archiveMutation
						.mutateAsync({ assetId, datasetId })
						.then(() => toast.success('Archived'))
						.catch((err: any) => toast.error(err?.message ?? 'Archive failed'));
				}}
			/>

			<AssetUpsertSheet
				open={open}
				onOpenChange={setOpen}
				mode={mode}
				asset={active}
				defaultDatasetId={datasetId}
				onSubmit={handleSubmit}
			/>
		</div>
	);
}
