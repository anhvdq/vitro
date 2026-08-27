import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow
} from '@/components/ui/table';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { FileDropZone } from '@/components/assets/file-drop-zone';
import { FileList } from '@/components/assets/file-list';
import { useAssetQuery } from '@/lib/assets/queries';
import { useAssetFilesQuery } from '@/lib/files/queries';
import { useAuthStore } from '@/lib/state/auth';
import { useBreadcrumbStore } from '@/lib/state/breadcrumbs';
import { createFileRoute } from '@tanstack/react-router';
import { useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';

export const Route = createFileRoute('/_protected/assets/$id')({
	head: () => ({
		meta: [{ title: 'Asset' }]
	}),
	component: RouteComponent,
	staticData: {
		breadcrumb: () => 'Asset'
	}
});

type PreviewRow = {
	order_id: string;
	order_date: string;
	customer_id: string;
	sku: string;
	qty: number;
	unit_price: number;
};

type SchemaField = {
	name: string;
	data_type: string;
	description: string;
	semantic_type: string;
	null_pct: number;
	stats: Record<string, string | number | undefined>;
	distribution?: { label: string; value: number }[];
};

function formatDate(iso: string) {
	const d = new Date(iso);
	if (Number.isNaN(d.getTime())) return iso;
	return d.toLocaleString();
}

function RouteComponent() {
	const { id } = Route.useParams();
	const hydrated = useAuthStore((s) => s.hydrated);
	const auth = useAuthStore((s) => s.auth);
	if (!hydrated || !auth) return null;

	return <AssetDetailPage assetId={id} />;
}

function AssetDetailPage({ assetId }: { assetId: string }) {
	const [tab, setTab] = useState('preview');
	const tabOptions = useMemo(
		() => [
			{ value: 'preview', label: 'Preview' },
			{ value: 'schema', label: 'Schema' },
			{ value: 'files', label: 'Files' },
			{ value: 'details', label: 'Details' }
		],
		[]
	);

	const assetQuery = useAssetQuery(assetId);
	const filesQuery = useAssetFilesQuery(assetId);

	useEffect(() => {
		if (!assetQuery.error) return;
		toast.error((assetQuery.error as any)?.message ?? 'Failed to load asset');
	}, [assetQuery.error]);

	useEffect(() => {
		if (!filesQuery.error) return;
		toast.error((filesQuery.error as any)?.message ?? 'Failed to load files');
	}, [filesQuery.error]);

	const asset = assetQuery.data;

	const previewRows: PreviewRow[] = useMemo(
		() => [
			{
				order_id: 'SO-0009821',
				order_date: '2025-03-22',
				customer_id: 'C-100381',
				sku: 'SKU-RED-SHOE-09',
				qty: 2,
				unit_price: 79.99
			},
			{
				order_id: 'SO-0009822',
				order_date: '2025-03-22',
				customer_id: 'C-100927',
				sku: 'SKU-BLK-JACKET-M',
				qty: 1,
				unit_price: 149.0
			},
			{
				order_id: 'SO-0009823',
				order_date: '2025-03-23',
				customer_id: 'C-100103',
				sku: 'SKU-GRN-HAT',
				qty: 3,
				unit_price: 24.5
			},
			{
				order_id: 'SO-0009824',
				order_date: '2025-03-23',
				customer_id: 'C-100564',
				sku: 'SKU-WHT-TSHIRT-L',
				qty: 1,
				unit_price: 32.0
			},
			{
				order_id: 'SO-0009825',
				order_date: '2025-03-24',
				customer_id: 'C-100220',
				sku: 'SKU-BLU-JEANS-32',
				qty: 2,
				unit_price: 89.5
			}
		],
		[]
	);

	const schemaFields = useMemo<SchemaField[]>(
		() => [
			{
				name: 'order_id',
				data_type: 'STRING',
				description: 'Unique sales order identifier.',
				semantic_type: 'nominal',
				null_pct: 0,
				stats: { distinct: 981_224, mode: 'SO-0009821' }
			},
			{
				name: 'order_date',
				data_type: 'DATE',
				description: 'Order creation date.',
				semantic_type: 'ordinal',
				null_pct: 0,
				stats: { min: '2024-01-01', max: '2025-03-24' }
			},
			{
				name: 'customer_id',
				data_type: 'STRING',
				description: 'Customer identifier.',
				semantic_type: 'nominal',
				null_pct: 0.2,
				stats: { distinct: 412_913, mode: 'C-100103' }
			},
			{
				name: 'sku',
				data_type: 'STRING',
				description: 'Product SKU code.',
				semantic_type: 'nominal',
				null_pct: 0,
				stats: { distinct: 18_420, mode: 'SKU-WHT-TSHIRT-L' },
				distribution: [
					{ label: 'Tops', value: 42 },
					{ label: 'Bottoms', value: 31 },
					{ label: 'Footwear', value: 18 },
					{ label: 'Accessories', value: 9 }
				]
			},
			{
				name: 'qty',
				data_type: 'INT',
				description: 'Quantity purchased for the line item.',
				semantic_type: 'ordinal',
				null_pct: 0,
				stats: { min: 1, max: 25, mean: 1.7, median: 1, mode: 1 },
				distribution: []
			},
			{
				name: 'unit_price',
				data_type: 'DECIMAL(10,2)',
				description: 'Unit price at time of purchase.',
				semantic_type: 'continuous',
				null_pct: 0.1,
				stats: { min: 0.99, max: 999.0, mean: 84.2, median: 62.5 },
				distribution: []
			}
		],
		[]
	);

	const setLabel = useBreadcrumbStore((s) => s.setLabel);
	useEffect(() => {
		if (!asset) return;
		setLabel(Route.id, asset.display_name);
	}, [asset?.display_name, setLabel]);

	if (!asset) return null;

	return (
		<div className="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
			<div className="flex flex-wrap items-center justify-between gap-3 px-4 lg:px-6">
				<div className="min-w-0">
					<div className="flex items-center gap-2">
						<h1 className="text-xl font-semibold">{asset.display_name}</h1>
						<Badge variant="outline" className="text-muted-foreground">
							{asset.status}
						</Badge>
					</div>
					<p className="text-muted-foreground mt-1 text-sm">{asset.description}</p>
				</div>
			</div>

			<Tabs className="px-4 lg:px-6" value={tab} onValueChange={setTab}>
				<div>
					<div className="flex flex-wrap items-center justify-between gap-3">
						<TabsList>
							{tabOptions.map((t) => (
								<TabsTrigger key={t.value} value={t.value}>
									{t.label}
								</TabsTrigger>
							))}
						</TabsList>
					</div>
				</div>

				<TabsContent value="preview" className="mt-4">
					<div className="overflow-hidden rounded-lg border">
						<Table>
							<TableHeader className="bg-muted sticky top-0 z-10">
								<TableRow>
									<TableHead>Order ID</TableHead>
									<TableHead>Date</TableHead>
									<TableHead>Customer</TableHead>
									<TableHead>SKU</TableHead>
									<TableHead className="text-right">Qty</TableHead>
									<TableHead className="text-right">Unit Price</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{previewRows.map((r) => (
									<TableRow key={r.order_id}>
										<TableCell className="font-mono text-xs">{r.order_id}</TableCell>
										<TableCell>{r.order_date}</TableCell>
										<TableCell className="font-mono text-xs">{r.customer_id}</TableCell>
										<TableCell className="font-mono text-xs">{r.sku}</TableCell>
										<TableCell className="text-right tabular-nums">{r.qty}</TableCell>
										<TableCell className="text-right tabular-nums">
											{r.unit_price.toFixed(2)}
										</TableCell>
									</TableRow>
								))}
							</TableBody>
						</Table>
					</div>
				</TabsContent>

				<TabsContent value="schema" className="mt-4">
					<SchemaTable fields={schemaFields} />
				</TabsContent>

				<TabsContent value="files" className="mt-4 flex flex-col gap-4">
					<FileDropZone assetId={asset.id} />
					<FileList assetId={asset.id} data={filesQuery.data ?? []} />
				</TabsContent>

				<TabsContent value="details" className="mt-4">
					<div className="rounded-lg border p-4">
						<div className="grid gap-4 md:grid-cols-2">
							<DetailItem label="Name (snake_case)" value={asset.name} monospace />
							<DetailItem label="Dataset ID" value={asset.dataset_id} monospace />
							<DetailItem label="Owner" value={asset.owner_user_name} />
							<DetailItem label="Connector Type" value={asset.connector.type} />
							<DetailItem label="Connector ID" value={asset.connector.id} monospace />
							<DetailItem label="Storage Prefix" value={asset.connector.storage_prefix} monospace />
						</div>

						<Separator className="my-4" />

						<div className="grid gap-4 md:grid-cols-2">
							<DetailItem label="Created At" value={formatDate(asset.created_at)} />
							<DetailItem label="Updated At" value={formatDate(asset.updated_at)} />
						</div>

						<Separator className="my-4" />

						<div className="grid gap-3">
							<div>
								<div className="text-muted-foreground text-xs">Allowed formats</div>
								<div className="mt-2 flex flex-wrap gap-2">
									{asset.connector.allowed_formats.map((f) => (
										<Badge key={f} variant="outline" className="text-muted-foreground">
											{f}
										</Badge>
									))}
								</div>
							</div>

							<div>
								<div className="text-muted-foreground text-xs">Tags</div>
								<div className="mt-2 flex flex-wrap gap-2">
									{asset.tags.map((t) => (
										<Badge key={t} variant="secondary">
											{t}
										</Badge>
									))}
								</div>
							</div>
						</div>
					</div>
				</TabsContent>
			</Tabs>
		</div>
	);
}

function DetailItem({
	label,
	value,
	monospace
}: {
	label: string;
	value: string;
	monospace?: boolean;
}) {
	return (
		<div>
			<div className="text-muted-foreground text-xs">{label}</div>
			<div
				className={['mt-1 wrap-break-word', monospace ? 'font-mono text-xs' : 'text-sm'].join(' ')}
			>
				{value}
			</div>
		</div>
	);
}

function SchemaTable({ fields }: { fields: SchemaField[] }) {
	return (
		<div className="overflow-hidden rounded-lg border">
			<Table>
				<TableHeader className="bg-muted sticky top-0 z-10">
					<TableRow>
						<TableHead>Field</TableHead>
						<TableHead>Type</TableHead>
						<TableHead>Description</TableHead>
						<TableHead className="text-right">Null %</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{fields.map((f) => (
						<TableRow key={f.name}>
							<TableCell className="font-mono text-xs">{f.name}</TableCell>
							<TableCell className="font-mono text-xs">{f.data_type}</TableCell>
							<TableCell className="text-sm">{f.description}</TableCell>
							<TableCell className="text-right tabular-nums">{f.null_pct}</TableCell>
						</TableRow>
					))}
				</TableBody>
			</Table>
		</div>
	);
}
