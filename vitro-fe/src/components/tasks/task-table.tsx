import { ColumnDef } from '@tanstack/react-table';
import { CheckCircle2Icon, LoaderIcon, MoreVerticalIcon, XCircleIcon } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';

import { DataTable } from '@/components/data-table';
import { Task, TaskSchema } from '@/lib/tasks/types';

function formatDate(iso: string) {
	const d = new Date(iso);
	if (Number.isNaN(d.getTime())) return iso;
	return d.toLocaleString();
}

function formatTaskType(taskType: string) {
	return taskType
		.split('_')
		.map((part) => (part ? part[0].toUpperCase() + part.slice(1).toLowerCase() : part))
		.join(' ');
}

function StatusBadge({ status }: { status: string }) {
	const normalized = status.toUpperCase();
	const isSuccess = normalized === 'SUCCEEDED' || normalized === 'COMPLETED';
	const isFailed = normalized === 'FAILED' || normalized === 'CANCELED';

	return (
		<Badge
			variant="outline"
			className="text-muted-foreground flex items-center gap-1 px-1.5 [&_svg]:size-3"
		>
			{isSuccess ? (
				<CheckCircle2Icon className="text-green-500 dark:text-green-400" />
			) : isFailed ? (
				<XCircleIcon className="text-destructive" />
			) : (
				<LoaderIcon />
			)}
			{normalized}
		</Badge>
	);
}

const baseColumns: ColumnDef<Task>[] = [
	{
		id: '_index',
		header: () => <div className="w-full text-left">#</div>,
		cell: ({ row }) => <div className="text-left">{row.index + 1}</div>,
		enableSorting: false,
		enableHiding: false
	},
	{
		accessorKey: 'task_type',
		header: 'Task',
		cell: ({ row }) => (
			<div className="text-foreground max-w-40 text-left font-medium">
				{formatTaskType(row.original.task_type)}
			</div>
		),
		enableHiding: false
	},
	{
		accessorKey: 'status',
		header: 'Status',
		cell: ({ row }) => <StatusBadge status={row.original.status} />
	},
	{
		accessorKey: 'asset_id',
		header: 'Asset',
		cell: ({ row }) => <div className="max-w-56 truncate">{row.original.asset_id}</div>
	},
	{
		accessorKey: 'created_by_name',
		header: 'Created By',
		cell: ({ row }) => <div className="max-w-40 truncate">{row.original.created_by_name}</div>
	},
	{
		accessorKey: 'correlation_id',
		header: 'Correlation',
		cell: ({ row }) => (
			<div className="max-w-56 truncate font-mono text-xs">{row.original.correlation_id}</div>
		)
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

export function TaskTable({
	data: initialData,
	onCreateTask,
	onViewTask
}: {
	data: Task[];
	onCreateTask?: () => void;
	onViewTask?: (task: Task) => void;
}) {
	const statuses = [
		{ label: 'QUEUED', value: 'QUEUED' },
		{ label: 'RUNNING', value: 'RUNNING' },
		{ label: 'SUCCEEDED', value: 'SUCCEEDED' },
		{ label: 'FAILED', value: 'FAILED' },
		{ label: 'CANCELED', value: 'CANCELED' }
	];

	const statusFilter = {
		columnId: 'status',
		label: 'Status',
		options: statuses
	};

	const actionColumnDef: ColumnDef<Task> = {
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
					{onViewTask && (
						<DropdownMenuItem onClick={() => onViewTask(row.original)}>View</DropdownMenuItem>
					)}
				</DropdownMenuContent>
			</DropdownMenu>
		)
	};

	const searchBarFilter = {
		columnId: 'task_type',
		placeholder: 'Find tasks...'
	};

	return (
		<DataTable<typeof TaskSchema>
			data={initialData}
			columns={[...baseColumns, actionColumnDef]}
			getRowId={(row) => row.id}
			searchBarFilter={searchBarFilter}
			facetedFilters={[statusFilter]}
			onCreate={onCreateTask}
		/>
	);
}
