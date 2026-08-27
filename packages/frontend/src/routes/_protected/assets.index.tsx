import { AssetTable } from '@/components/assets/asset-table';
import { AssetUpsertInput, AssetUpsertSheet } from '@/components/assets/form';
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue
} from '@/components/ui/select';
import { Asset } from '@/lib/assets/types';
import {
	useArchiveAssetMutation,
	useAssetsQuery,
	useCreateAssetMutation,
	useUpdateAssetMutation
} from '@/lib/assets/queries';
import { useDatasetsQuery } from '@/lib/datasets/queries';
import { useAuthStore } from '@/lib/state/auth';
import { useUiStore } from '@/lib/state/ui';
import { createFileRoute, useRouter } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

export const Route = createFileRoute('/_protected/assets/')({
	head: () => ({
		meta: [{ title: 'Assets' }]
	}),
	component: RouteComponent
});

type SheetMode = 'create' | 'edit' | 'view';

const ALL_DATASETS = '__all__';

function RouteComponent() {
	const hydrated = useAuthStore((s) => s.hydrated);
	const auth = useAuthStore((s) => s.auth);
	if (!hydrated || !auth) return null;

	return <AssetsPage />;
}

function AssetsPage() {
	const router = useRouter();
	const datasetsQuery = useDatasetsQuery();
	const selectedDatasetId = useUiStore((s) => s.selectedDatasetId);
	const setSelectedDatasetId = useUiStore((s) => s.setSelectedDatasetId);

	const [open, setOpen] = useState(false);
	const [mode, setMode] = useState<SheetMode>('create');
	const [active, setActive] = useState<Asset | null>(null);

	useEffect(() => {
		if (!datasetsQuery.error) return;
		toast.error((datasetsQuery.error as any)?.message ?? 'Failed to load datasets');
	}, [datasetsQuery.error]);

	const datasets = datasetsQuery.data;
	if (!datasets) return null;

	return (
		<div className="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
			<div className="flex flex-wrap items-center gap-3 px-4 lg:px-6">
				<div className="w-full max-w-sm">
					<Select
						value={selectedDatasetId ?? ALL_DATASETS}
						onValueChange={(value) => setSelectedDatasetId(value === ALL_DATASETS ? null : value)}
					>
						<SelectTrigger>
							<SelectValue placeholder="Select a dataset" />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value={ALL_DATASETS}>All datasets</SelectItem>
							{datasets.map((d) => (
								<SelectItem key={d.id} value={d.id}>
									{d.display_name}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</div>
			</div>

			<AssetsForDataset
				datasetId={selectedDatasetId ?? undefined}
				open={open}
				onOpenChange={setOpen}
				mode={mode}
				setMode={setMode}
				active={active}
				setActive={setActive}
				onViewAsset={(assetId) => router.navigate({ to: '/assets/$id', params: { id: assetId } })}
			/>
		</div>
	);
}

function AssetsForDataset({
	datasetId,
	open,
	onOpenChange,
	mode,
	setMode,
	active,
	setActive,
	onViewAsset
}: {
	datasetId?: string;
	open: boolean;
	onOpenChange: (open: boolean) => void;
	mode: SheetMode;
	setMode: (mode: SheetMode) => void;
	active: Asset | null;
	setActive: (asset: Asset | null) => void;
	onViewAsset: (assetId: string) => void;
}) {
	const assetsQuery = useAssetsQuery(datasetId);
	const createMutation = useCreateAssetMutation();
	const updateMutation = useUpdateAssetMutation();
	const archiveMutation = useArchiveAssetMutation();

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
			datasetId: payload.dataset_id,
			input: {
				display_name: payload.display_name,
				description: payload.description,
				tags: payload.tags
			}
		});
	}

	const assets = assetsQuery.data;
	if (!assets) return null;

	return (
		<>
			<AssetTable
				data={assets}
				onCreateAsset={() => {
					setMode('create');
					setActive(null);
					onOpenChange(true);
				}}
				createDisabled={!datasetId}
				createDisabledReason={!datasetId ? 'Select a dataset to create an asset in' : undefined}
				onViewAsset={(asset) => onViewAsset(asset.id)}
				onEditAsset={(asset) => {
					setMode('edit');
					setActive(asset);
					onOpenChange(true);
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
				onOpenChange={onOpenChange}
				mode={mode}
				asset={active}
				defaultDatasetId={datasetId}
				onSubmit={handleSubmit}
			/>
		</>
	);
}
