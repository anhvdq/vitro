import { ColumnDef } from '@tanstack/react-table';
import { CheckCircle2Icon, LoaderIcon, MoreVerticalIcon } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import { DataTable } from '@/components/data-table';
import { Asset, AssetSchema } from '@/lib/assets/types';
import { Separator } from '@/components/ui/separator';

function formatDate(iso: string) {
	const d = new Date(iso);
	if (Number.isNaN(d.getTime())) return iso;
	return d.toLocaleString();
}

function StatusBadge({ status }: { status: string }) {
	const normalized = status.toUpperCase();

	const isActive = normalized === 'ACTIVE';
	const isArchived = normalized === 'ARCHIVED';

	return (
		<Badge
			variant="outline"
			className="text-muted-foreground flex items-center gap-1 px-1.5 [&_svg]:size-3"
		>
			{isActive ? (
				<CheckCircle2Icon className="text-green-500 dark:text-green-400" />
			) : (
				<LoaderIcon className={isArchived ? 'text-muted-foreground' : undefined} />
			)}
			{normalized}
		</Badge>
	);
}

const baseColumns: ColumnDef<Asset>[] = [
	{
		id: '_index',
		header: () => <div className="w-full text-left">#</div>,
		cell: ({ row }) => <div className="text-left">{row.index + 1}</div>,
		enableSorting: false,
		enableHiding: false
	},
	{
		accessorKey: 'display_name',
		header: 'Name',
		cell: ({ row }) => (
			<div className="text-foreground max-w-40 text-left font-medium">
				{row.original.display_name}
			</div>
		),
		enableHiding: false
	},
	{
		accessorKey: 'description',
		header: 'Description',
		cell: ({ row }) => {
			const description = row.original.description;
			if (!description) return <span className="text-muted-foreground">—</span>;

			return (
				<div className="max-w-44 truncate" title={description}>
					{description}
				</div>
			);
		}
	},
	{
		accessorKey: 'owner_user_name',
		header: 'Owner',
		cell: ({ row }) => <div className="max-w-36 truncate">{row.original.owner_user_name}</div>
	},
	{
		accessorKey: 'tags',
		header: 'Tags',
		cell: ({ row }) => {
			const tags = row.original.tags ?? [];
			if (!tags.length) return <span className="text-muted-foreground">—</span>;

			const shown = tags.slice(0, 2);
			const rest = tags.length - shown.length;

			return (
				<div className="flex flex-wrap gap-1">
					{shown.map((t) => (
						<Badge key={t} variant="secondary" className="px-1.5">
							{t}
						</Badge>
					))}
					{rest > 0 ? (
						<Badge variant="outline" className="text-muted-foreground px-1.5">
							+{rest}
						</Badge>
					) : null}
				</div>
			);
		}
	},
	{
		accessorKey: 'connector.type',
		header: 'Connector',
		cell: ({ row }) => (
			<Badge variant="outline" className="text-muted-foreground px-1.5">
				{row.original.connector.type}
			</Badge>
		)
	},
	{
		accessorKey: 'connector.allowed_formats',
		header: 'Formats',
		cell: ({ row }) => {
			const formats = row.original.connector.allowed_formats ?? [];
			if (!formats.length) return <span className="text-muted-foreground">—</span>;

			return (
				<div className="flex flex-wrap gap-1">
					{formats.map((format) => (
						<Badge key={format} variant="outline" className="text-muted-foreground px-1.5">
							{format}
						</Badge>
					))}
				</div>
			);
		}
	},
	{
		accessorKey: 'status',
		header: 'Status',
		cell: ({ row }) => <StatusBadge status={row.original.status} />
	},
	{
		accessorKey: 'updated_at',
		header: () => <div className="w-full text-right">Updated</div>,
		meta: {
			displayName: 'Updated'
		},
		cell: ({ row }) => (
			<div className="text-muted-foreground w-full text-right tabular-nums">
				{formatDate(row.original.updated_at)}
			</div>
		)
	}
];

export function AssetTable({
	data: initialData,
	onCreateAsset,
	onViewAsset,
	onEditAsset,
	onDeleteAsset,
	createDisabled,
	createDisabledReason
}: {
	data: Asset[];
	onCreateAsset?: () => void;
	onViewAsset?: (asset: Asset) => void;
	onEditAsset?: (asset: Asset) => void;
	onDeleteAsset?: (id: string) => void;
	createDisabled?: boolean;
	createDisabledReason?: string;
}) {
	const statuses = [
		{ label: 'ACTIVE', value: 'ACTIVE' },
		{ label: 'ARCHIVED', value: 'ARCHIVED' }
	];

	const statusFilter = {
		columnId: 'status',
		label: 'Status',
		options: statuses
	};

	const actionColumnDef: ColumnDef<Asset> = {
		id: 'actions',
		cell: ({ row }) => (
			<DropdownMenu>
				<DropdownMenuTrigger asChild>
					<Button
						variant="ghost"
						className="text-muted-foreground data-[state=open]:bg-muted flex size-8"
						size="icon"
					>
						<MoreVerticalIcon />
						<span className="sr-only">Open menu</span>
					</Button>
				</DropdownMenuTrigger>

				<DropdownMenuContent align="end" className="w-32">
					{onViewAsset && (
						<DropdownMenuItem onClick={() => onViewAsset(row.original)}>View</DropdownMenuItem>
					)}
					{onEditAsset && (
						<DropdownMenuItem onClick={() => onEditAsset(row.original)}>Edit</DropdownMenuItem>
					)}
					{onDeleteAsset && (
						<>
							<Separator />
							<DropdownMenuItem
								onClick={() => onDeleteAsset(row.original.id)}
								className="text-destructive focus:text-destructive"
							>
								Delete
							</DropdownMenuItem>
						</>
					)}
				</DropdownMenuContent>
			</DropdownMenu>
		)
	};

	const searchBarFilter = {
		columnId: 'display_name',
		placeholder: 'Find assets...'
	};

	return (
		<DataTable<typeof AssetSchema>
			data={initialData}
			columns={[...baseColumns, actionColumnDef]}
			getRowId={(row) => row.id}
			searchBarFilter={searchBarFilter}
			facetedFilters={[statusFilter]}
			onCreate={onCreateAsset}
			createDisabled={createDisabled}
			createDisabledReason={createDisabledReason}
		/>
	);
}
