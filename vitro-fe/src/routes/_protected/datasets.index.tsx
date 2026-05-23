import { DatasetTable } from '@/components/datasets/dataset-table';
import { DatasetUpsertInput, DatasetUpsertSheet } from '@/components/datasets/form';
import { Dataset } from '@/lib/datasets/types';
import {
	useArchiveDatasetMutation,
	useCreateDatasetMutation,
	useDatasetsQuery,
	useUpdateDatasetMutation
} from '@/lib/datasets/queries';
import { useAuthStore } from '@/lib/state/auth';
import { createFileRoute, useRouter } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

export const Route = createFileRoute('/_protected/datasets/')({
	head: () => ({
		meta: [{ title: 'Datasets' }]
	}),
	component: RouteComponent,
	staticData: {
		breadcrumb: 'Datasets'
	}
});

type SheetMode = 'create' | 'edit' | 'view';

function RouteComponent() {
	const hydrated = useAuthStore((s) => s.hydrated);
	const auth = useAuthStore((s) => s.auth);
	if (!hydrated || !auth) return null;

	return <DatasetsPage />;
}

function DatasetsPage() {
	const router = useRouter();
	const datasetsQuery = useDatasetsQuery();
	const createMutation = useCreateDatasetMutation();
	const updateMutation = useUpdateDatasetMutation();
	const archiveMutation = useArchiveDatasetMutation();

	const [open, setOpen] = useState(false);
	const [mode, setMode] = useState<SheetMode>('create');
	const [active, setActive] = useState<Dataset | null>(null);

	useEffect(() => {
		if (!datasetsQuery.error) return;
		toast.error((datasetsQuery.error as any)?.message ?? 'Failed to load datasets');
	}, [datasetsQuery.error]);

	async function handleSubmit(
		payload: DatasetUpsertInput,
		ctx: { mode: SheetMode; datasetId?: string }
	) {
		if (ctx.mode === 'create') {
			await createMutation.mutateAsync(payload);
			return;
		}

		if (!ctx.datasetId) throw new Error('Missing dataset id');

		await updateMutation.mutateAsync({
			datasetId: ctx.datasetId,
			input: {
				display_name: payload.display_name,
				description: payload.description,
				tags: payload.tags
			}
		});
	}

	const datasets = datasetsQuery.data;
	if (!datasets) return null;

	return (
		<div className="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
			<DatasetTable
				data={datasets}
				onCreateDataset={() => {
					setMode('create');
					setActive(null);
					setOpen(true);
				}}
				onViewDataset={(dataset) => {
					router.navigate({ to: '/datasets/$id', params: { id: dataset.id } });
				}}
				onEditDataset={(dataset) => {
					setMode('edit');
					setActive(dataset);
					setOpen(true);
				}}
				onDeleteDataset={(id) => {
					archiveMutation
						.mutateAsync(id)
						.then(() => toast.success('Archived'))
						.catch((err: any) => toast.error(err?.message ?? 'Archive failed'));
				}}
			/>

			<DatasetUpsertSheet
				open={open}
				onOpenChange={setOpen}
				mode={mode}
				dataset={active}
				onSubmit={handleSubmit}
			/>
		</div>
	);
}
