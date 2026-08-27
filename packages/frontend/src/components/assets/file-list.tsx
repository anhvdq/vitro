import { ColumnDef } from '@tanstack/react-table';
import { DownloadIcon } from 'lucide-react';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import { DataTable } from '@/components/data-table';
import { downloadAssetFile } from '@/lib/files/api';
import { FileResponseDTO, FileResponseDTOSchema } from '@/lib/files/types';
import { useAuthStore } from '@/lib/state/auth';

function formatDate(iso: string) {
	const d = new Date(iso);
	if (Number.isNaN(d.getTime())) return iso;
	return d.toLocaleString();
}

function formatBytes(bytes: number) {
	if (bytes === 0) return '0 B';
	const units = ['B', 'KB', 'MB', 'GB', 'TB'];
	const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
	const value = bytes / 1024 ** exponent;
	return `${exponent === 0 ? value : value.toFixed(1)} ${units[exponent]}`;
}

const columns: ColumnDef<FileResponseDTO>[] = [
	{
		accessorKey: 'file_name',
		header: 'Name',
		cell: ({ row }) => (
			<div className="text-foreground max-w-64 truncate font-mono text-xs">
				{row.original.file_name}
			</div>
		),
		enableHiding: false
	},
	{
		accessorKey: 'size_bytes',
		header: () => <div className="w-full text-right">Size</div>,
		cell: ({ row }) => (
			<div className="text-muted-foreground w-full text-right tabular-nums">
				{formatBytes(row.original.size_bytes)}
			</div>
		)
	},
	{
		accessorKey: 'content_type',
		header: 'Type',
		cell: ({ row }) => (
			<div className="text-muted-foreground text-xs">{row.original.content_type ?? '—'}</div>
		)
	},
	{
		accessorKey: 'created_at',
		header: () => <div className="w-full text-right">Uploaded</div>,
		meta: {
			displayName: 'Uploaded'
		},
		cell: ({ row }) => (
			<div className="text-muted-foreground w-full text-right tabular-nums">
				{formatDate(row.original.created_at)}
			</div>
		)
	}
];

export function FileList({ assetId, data }: { assetId: string; data: FileResponseDTO[] }) {
	const auth = useAuthStore((s) => s.auth)!;

	const actionColumnDef: ColumnDef<FileResponseDTO> = {
		id: 'actions',
		header: () => <div className="w-full text-right">Actions</div>,
		cell: ({ row }) => (
			<div className="flex justify-end">
				<Button
					variant="ghost"
					size="icon"
					className="size-8"
					onClick={() => {
						downloadAssetFile({
							accessToken: auth.accessToken,
							workspaceId: auth.workspaceId,
							assetId,
							fileId: row.original.id,
							fileName: row.original.file_name
						}).catch((err: unknown) =>
							toast.error(err instanceof Error ? err.message : 'Download failed')
						);
					}}
				>
					<DownloadIcon />
					<span className="sr-only">Download</span>
				</Button>
			</div>
		)
	};

	return (
		<DataTable<typeof FileResponseDTOSchema>
			data={data}
			columns={[...columns, actionColumnDef]}
			getRowId={(row) => row.id}
			searchBarFilter={{ columnId: 'file_name', placeholder: 'Find files...' }}
		/>
	);
}
