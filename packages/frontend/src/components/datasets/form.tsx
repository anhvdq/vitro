import { toast } from 'sonner';
import { X } from 'lucide-react';

import { Dataset } from '@/lib/datasets/types';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Separator } from '@/components/ui/separator';
import { Textarea } from '@/components/ui/textarea';
import {
	Sheet,
	SheetClose,
	SheetContent,
	SheetDescription,
	SheetFooter,
	SheetHeader,
	SheetTitle
} from '@/components/ui/sheet';
import { useState } from 'react';

type SheetMode = 'create' | 'edit' | 'view';

export type DatasetUpsertInput = {
	name: string;
	display_name: string;
	description: string;
	tags: string[];
};

function normalizeTag(tag: string) {
	return tag.trim().replace(/\s+/g, '-');
}

function dedupeTags(tags: string[]) {
	const seen = new Set<string>();
	const out: string[] = [];
	for (const t of tags) {
		const k = t.toLowerCase();
		if (!k) continue;
		if (seen.has(k)) continue;
		seen.add(k);
		out.push(t);
	}
	return out;
}

export function DatasetUpsertSheet({
	open,
	onOpenChange,
	mode,
	dataset,
	onSubmit
}: {
	open: boolean;
	onOpenChange: (v: boolean) => void;
	mode: SheetMode;
	dataset: Dataset | null;
	onSubmit?: (
		input: DatasetUpsertInput,
		ctx: { mode: SheetMode; datasetId?: string }
	) => Promise<void> | void;
}) {
	const readOnly = mode === 'view';
	const formKey = `${mode}:${dataset?.id ?? 'new'}`;

	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent side="right" className="flex flex-col">
				<SheetHeader className="gap-1">
					<SheetTitle>
						{mode === 'create'
							? 'Create dataset'
							: mode === 'edit'
								? 'Edit dataset'
								: 'Dataset details'}
					</SheetTitle>
					<SheetDescription>
						{mode === 'create'
							? 'Add a dataset to this workspace.'
							: mode === 'edit'
								? 'Update fields and save changes.'
								: 'View dataset information.'}
					</SheetDescription>
				</SheetHeader>

				<DatasetUpsertForm
					key={formKey}
					mode={mode}
					dataset={dataset}
					readOnly={readOnly}
					onSubmit={onSubmit}
					close={() => onOpenChange(false)}
				/>
			</SheetContent>
		</Sheet>
	);
}

function DatasetUpsertForm({
	mode,
	dataset,
	readOnly,
	onSubmit,
	close
}: {
	mode: SheetMode;
	dataset: Dataset | null;
	readOnly: boolean;
	onSubmit?: (
		input: DatasetUpsertInput,
		ctx: { mode: SheetMode; datasetId?: string }
	) => Promise<void> | void;
	close: () => void;
}) {
	const [name, setName] = useState(dataset?.name ?? '');
	const [displayName, setDisplayName] = useState(dataset?.display_name ?? '');
	const [description, setDescription] = useState(dataset?.description ?? '');

	// chips state
	const [tags, setTags] = useState<string[]>(dataset?.tags ?? []);
	const [tagInput, setTagInput] = useState('');

	function addTag(raw: string) {
		const normalized = normalizeTag(raw);
		if (!normalized) return;

		setTags((prev) => dedupeTags([...prev, normalized]));
		setTagInput('');
	}

	function removeTag(tag: string) {
		setTags((prev) => prev.filter((t) => t.toLowerCase() !== tag.toLowerCase()));
	}

	async function handleSubmit() {
		const input: DatasetUpsertInput = {
			name: name.trim(),
			display_name: displayName.trim(),
			description: description.trim(),
			tags
		};

		try {
			await onSubmit?.(input, { mode, datasetId: dataset?.id });
			toast.success(mode === 'create' ? 'Created' : 'Saved');
			close();
		} catch {
			toast.error(mode === 'create' ? 'Create failed' : 'Update failed');
		}
	}

	return (
		<div className="flex flex-1 flex-col gap-4 overflow-y-auto py-4 text-sm">
			<div className="flex flex-col gap-3">
				<Label htmlFor="ds-name">Name (snake_case)</Label>
				<Input
					id="ds-name"
					value={name}
					onChange={(e) => setName(e.target.value)}
					disabled={readOnly || mode !== 'create'}
					placeholder="e.g. sales_data"
				/>
			</div>

			<div className="flex flex-col gap-3">
				<Label htmlFor="ds-display-name">Display name</Label>
				<Input
					id="ds-display-name"
					value={displayName}
					onChange={(e) => setDisplayName(e.target.value)}
					disabled={readOnly}
					placeholder="e.g. Sales Data"
				/>
			</div>

			{/* Description: multiline */}
			<div className="flex flex-col gap-3">
				<Label htmlFor="ds-description">Description</Label>
				<Textarea
					id="ds-description"
					value={description}
					onChange={(e) => setDescription(e.target.value)}
					disabled={readOnly}
					placeholder="Describe this dataset..."
					className="min-h-24 resize-y"
				/>
			</div>

			{/* Tags: chips */}
			<div className="flex flex-col gap-3">
				<Label htmlFor="ds-tags">Tags</Label>

				<div
					className={[
						'flex min-h-10 flex-wrap items-center gap-2 rounded-md border bg-transparent px-3 py-2',
						readOnly ? 'opacity-70' : 'focus-within:ring-ring focus-within:ring-1'
					].join(' ')}
				>
					{tags.length ? (
						tags.map((t) => (
							<span
								key={t}
								className="bg-muted text-foreground inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs"
							>
								{t}
								{!readOnly ? (
									<button
										type="button"
										onClick={() => removeTag(t)}
										className="text-muted-foreground hover:text-foreground"
										aria-label={`Remove tag ${t}`}
									>
										<X className="h-3 w-3" />
									</button>
								) : null}
							</span>
						))
					) : (
						<span className="text-muted-foreground text-xs">No tags</span>
					)}

					{!readOnly ? (
						<input
							id="ds-tags"
							value={tagInput}
							onChange={(e) => setTagInput(e.target.value)}
							onKeyDown={(e) => {
								if (e.key === 'Enter' || e.key === ',') {
									e.preventDefault();
									addTag(tagInput);
								}

								if (e.key === 'Backspace' && tagInput.length === 0 && tags.length > 0) {
									// remove last chip
									e.preventDefault();
									setTags((prev) => prev.slice(0, -1));
								}
							}}
							placeholder="Type a tag and press Enter"
							className="placeholder:text-muted-foreground min-w-40 flex-1 border-transparent bg-transparent text-sm outline-none focus:border-transparent focus:ring-0"
						/>
					) : null}
				</div>

				{!readOnly ? (
					<p className="text-muted-foreground text-xs">
						Press Enter (or comma) to add. Backspace removes the last tag.
					</p>
				) : null}
			</div>

			{dataset ? (
				<>
					<Separator />
					<div className="text-muted-foreground grid gap-2 text-xs">
						<div className="flex justify-between gap-3">
							<span>Owner</span>
							<span className="text-foreground">{dataset.owner_user_name}</span>
						</div>
						<div className="flex justify-between gap-3">
							<span>Created</span>
							<span className="text-foreground">
								{new Date(dataset.created_at).toLocaleString()}
							</span>
						</div>
						<div className="flex justify-between gap-3">
							<span>Updated</span>
							<span className="text-foreground">
								{new Date(dataset.updated_at).toLocaleString()}
							</span>
						</div>
						{dataset.archived_at ? (
							<div className="flex justify-between gap-3">
								<span>Archived</span>
								<span className="text-foreground">
									{new Date(dataset.archived_at).toLocaleString()}
								</span>
							</div>
						) : null}
					</div>
				</>
			) : null}

			<SheetFooter className="mt-auto flex gap-2 sm:flex-col sm:space-x-0">
				{mode !== 'view' ? (
					<Button className="w-full" onClick={handleSubmit}>
						{mode === 'create' ? 'Create' : 'Save changes'}
					</Button>
				) : null}

				<SheetClose asChild>
					<Button variant="outline" className="w-full">
						Done
					</Button>
				</SheetClose>
			</SheetFooter>
		</div>
	);
}
