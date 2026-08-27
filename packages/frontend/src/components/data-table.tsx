import * as React from 'react';

import { SortableContext, useSortable, verticalListSortingStrategy } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import {
	ColumnDef,
	ColumnFiltersState,
	Row,
	SortingState,
	VisibilityState,
	flexRender,
	getCoreRowModel,
	getFacetedRowModel,
	getFacetedUniqueValues,
	getFilteredRowModel,
	getPaginationRowModel,
	getSortedRowModel,
	useReactTable
} from '@tanstack/react-table';
import {
	ChevronDownIcon,
	ChevronLeftIcon,
	ChevronRightIcon,
	ChevronsLeftIcon,
	ChevronsRightIcon,
	ColumnsIcon,
	PlusIcon
} from 'lucide-react';

import { z } from 'zod';

import { Button } from '@/components/ui/button';

import {
	DropdownMenu,
	DropdownMenuCheckboxItem,
	DropdownMenuContent,
	DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue
} from '@/components/ui/select';
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow
} from '@/components/ui/table';
import {
	DataTableFacetedFilter,
	FacetedFilterOption
} from '@/components/data-table-faceted-filter';
import { UniqueIdentifier } from '@dnd-kit/core';
import { ColumnDefCustomMeta } from '@/lib/types';

function DataRow<T extends z.ZodObject>({
	row,
	getRowId
}: {
	row: Row<z.infer<T>>;
	getRowId: (row: z.infer<T>) => string;
}) {
	const { transform, transition, setNodeRef } = useSortable({
		id: getRowId(row.original)
	});

	return (
		<TableRow
			data-state={row.getIsSelected() && 'selected'}
			ref={setNodeRef}
			className="relative z-0"
			style={{
				transform: CSS.Transform.toString(transform),
				transition: transition
			}}
		>
			{row.getVisibleCells().map((cell) => (
				<TableCell key={cell.id}>
					{flexRender(cell.column.columnDef.cell, cell.getContext())}
				</TableCell>
			))}
		</TableRow>
	);
}

type FacetedFilter = {
	columnId: string;
	label: string;
	options: FacetedFilterOption[];
};

type SearchBarFilter = {
	columnId: string;
	placeholder: string;
};

export function DataTable<T extends z.ZodObject>({
	data: initialData,
	columns,
	getRowId,
	searchBarFilter,
	facetedFilters,
	onCreate,
	createDisabled,
	createDisabledReason
}: {
	data: z.infer<T>[];
	columns: ColumnDef<z.infer<T>>[];
	getRowId: (row: z.infer<T>) => string;
	searchBarFilter?: SearchBarFilter;
	facetedFilters?: FacetedFilter[];
	onCreate?: () => void;
	createDisabled?: boolean;
	createDisabledReason?: string;
}) {
	const [data, setData] = React.useState(() => initialData);
	const [rowSelection, setRowSelection] = React.useState({});
	const [columnVisibility, setColumnVisibility] = React.useState<VisibilityState>({});
	const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>([]);
	const [sorting, setSorting] = React.useState<SortingState>([]);
	const [pagination, setPagination] = React.useState({
		pageIndex: 0,
		pageSize: 10
	});

	React.useEffect(() => {
		setData(initialData);
	}, [initialData]);

	const dataIds = React.useMemo<UniqueIdentifier[]>(
		() => data?.map((row) => getRowId(row)) || [],
		[data, getRowId]
	);

	const table = useReactTable({
		data,
		columns,
		state: {
			sorting,
			columnVisibility,
			rowSelection,
			columnFilters,
			pagination
		},
		getRowId: getRowId,
		enableRowSelection: true,
		onRowSelectionChange: setRowSelection,
		onSortingChange: setSorting,
		onColumnFiltersChange: setColumnFilters,
		onColumnVisibilityChange: setColumnVisibility,
		onPaginationChange: setPagination,
		getCoreRowModel: getCoreRowModel(),
		getFilteredRowModel: getFilteredRowModel(),
		getPaginationRowModel: getPaginationRowModel(),
		getSortedRowModel: getSortedRowModel(),
		getFacetedRowModel: getFacetedRowModel(),
		getFacetedUniqueValues: getFacetedUniqueValues()
	});

	return (
		<>
			<div className="flex items-center justify-between px-4 lg:px-6">
				<div className="flex flex-1 items-center gap-2">
					{searchBarFilter && (
						<Input
							placeholder={searchBarFilter.placeholder}
							value={(table.getColumn(searchBarFilter.columnId)?.getFilterValue() as string) ?? ''}
							onChange={(event) =>
								table.getColumn(searchBarFilter.columnId)?.setFilterValue(event.target.value)
							}
							className="h-8 w-37.5 lg:w-62.5"
						/>
					)}
					{facetedFilters?.map((filter) => (
						<DataTableFacetedFilter
							key={filter.columnId}
							column={table.getColumn(filter.columnId)}
							title={filter.label}
							options={filter.options}
						/>
					))}
				</div>
				<div className="flex items-center gap-2">
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button variant="outline" size="sm">
								<ColumnsIcon />
								<span className="hidden lg:inline">Columns</span>
								<span className="lg:hidden">Columns</span>
								<ChevronDownIcon />
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end" className="w-56">
							{table
								.getAllColumns()
								.filter((column) => typeof column.accessorFn !== 'undefined' && column.getCanHide())
								.map((column) => {
									return (
										<DropdownMenuCheckboxItem
											key={column.id}
											className="capitalize"
											checked={column.getIsVisible()}
											onCheckedChange={(value) => column.toggleVisibility(!!value)}
										>
											{typeof column.columnDef.header === 'string'
												? column.columnDef.header
												: (column.columnDef.meta as ColumnDefCustomMeta).displayName}
										</DropdownMenuCheckboxItem>
									);
								})}
						</DropdownMenuContent>
					</DropdownMenu>
					{onCreate && (
						<Button
							variant="outline"
							size="sm"
							onClick={onCreate}
							disabled={createDisabled}
							title={createDisabled ? createDisabledReason : undefined}
						>
							<PlusIcon />
							<span className="hidden lg:inline">New</span>
						</Button>
					)}
				</div>
			</div>
			<div className="relative flex flex-col gap-4 overflow-auto px-4 lg:px-6">
				<div className="overflow-hidden rounded-lg border">
					<Table>
						<TableHeader className="bg-muted sticky top-0 z-10">
							{table.getHeaderGroups().map((headerGroup) => (
								<TableRow key={headerGroup.id}>
									{headerGroup.headers.map((header) => {
										return (
											<TableHead key={header.id} colSpan={header.colSpan}>
												{header.isPlaceholder
													? null
													: flexRender(header.column.columnDef.header, header.getContext())}
											</TableHead>
										);
									})}
								</TableRow>
							))}
						</TableHeader>
						<TableBody className="**:data-[slot=table-cell]:first:w-8">
							{table.getRowModel().rows?.length ? (
								<SortableContext items={dataIds} strategy={verticalListSortingStrategy}>
									{table.getRowModel().rows.map((row) => (
										<DataRow key={row.id} row={row} getRowId={getRowId} />
									))}
								</SortableContext>
							) : (
								<TableRow>
									<TableCell colSpan={columns.length} className="h-24 text-center">
										No results.
									</TableCell>
								</TableRow>
							)}
						</TableBody>
					</Table>
				</div>
				<div className="flex items-center justify-between px-4">
					<div className="text-muted-foreground hidden flex-1 text-sm lg:flex">
						{table.getFilteredSelectedRowModel().rows.length} of{' '}
						{table.getFilteredRowModel().rows.length} row(s) selected.
					</div>
					<div className="flex w-full items-center gap-8 lg:w-fit">
						<div className="hidden items-center gap-2 lg:flex">
							<Label htmlFor="rows-per-page" className="text-sm font-medium">
								Rows per page
							</Label>
							<Select
								value={`${table.getState().pagination.pageSize}`}
								onValueChange={(value) => {
									table.setPageSize(Number(value));
								}}
							>
								<SelectTrigger className="w-20" id="rows-per-page">
									<SelectValue placeholder={table.getState().pagination.pageSize} />
								</SelectTrigger>
								<SelectContent side="top">
									{[10, 20, 30, 40, 50].map((pageSize) => (
										<SelectItem key={pageSize} value={`${pageSize}`}>
											{pageSize}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</div>
						<div className="flex w-fit items-center justify-center text-sm font-medium">
							Page {table.getState().pagination.pageIndex + 1} of {table.getPageCount()}
						</div>
						<div className="ml-auto flex items-center gap-2 lg:ml-0">
							<Button
								variant="outline"
								className="hidden h-8 w-8 p-0 lg:flex"
								onClick={() => table.setPageIndex(0)}
								disabled={!table.getCanPreviousPage()}
							>
								<span className="sr-only">Go to first page</span>
								<ChevronsLeftIcon />
							</Button>
							<Button
								variant="outline"
								className="size-8"
								size="icon"
								onClick={() => table.previousPage()}
								disabled={!table.getCanPreviousPage()}
							>
								<span className="sr-only">Go to previous page</span>
								<ChevronLeftIcon />
							</Button>
							<Button
								variant="outline"
								className="size-8"
								size="icon"
								onClick={() => table.nextPage()}
								disabled={!table.getCanNextPage()}
							>
								<span className="sr-only">Go to next page</span>
								<ChevronRightIcon />
							</Button>
							<Button
								variant="outline"
								className="hidden size-8 lg:flex"
								size="icon"
								onClick={() => table.setPageIndex(table.getPageCount() - 1)}
								disabled={!table.getCanNextPage()}
							>
								<span className="sr-only">Go to last page</span>
								<ChevronsRightIcon />
							</Button>
						</div>
					</div>
				</div>
			</div>
		</>
	);
}
