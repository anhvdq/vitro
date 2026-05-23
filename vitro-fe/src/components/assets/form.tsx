import { toast } from 'sonner';
import { X } from 'lucide-react';
import { useState } from 'react';

import { Asset } from '@/lib/assets/types';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
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
import { Separator } from '@/components/ui/separator';

type SheetMode = 'create' | 'edit' | 'view';

export type AssetUpsertInput = {
	name: string;
	display_name: string;
	description: string;
	dataset_id: string;
	tags: string[];
};

function normalizeTag(tag: string) {
	return tag.trim().replace(/\s+/g, '-');
}

function dedupeTokens(tokens: string[]) {
	const seen = new Set<string>();
	const out: string[] = [];
	for (const token of tokens) {
		const key = token.toLowerCase();
		if (!key) continue;
		if (seen.has(key)) continue;
		seen.add(key);
		out.push(token);
	}
	return out;
}

export function AssetUpsertSheet({
	open,
	onOpenChange,
	mode,
	asset,
	defaultDatasetId,
	onSubmit
}: {
	open: boolean;
	onOpenChange: (v: boolean) => void;
	mode: SheetMode;
	asset: Asset | null;
	defaultDatasetId: string;
	onSubmit?: (
		input: AssetUpsertInput,
		ctx: { mode: SheetMode; assetId?: string }
	) => Promise<void> | void;
}) {
	const readOnly = mode === 'view';
	const formKey = `${mode}:${asset?.id ?? 'new'}`;

	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent side="right" className="flex flex-col">
				<SheetHeader className="gap-1">
					<SheetTitle>
						{mode === 'create' ? 'Create asset' : mode === 'edit' ? 'Edit asset' : 'Asset details'}
					</SheetTitle>
					<SheetDescription>
						{mode === 'create'
							? 'Add an asset to this workspace.'
							: mode === 'edit'
								? 'Update fields and save changes.'
								: 'View asset information.'}
					</SheetDescription>
				</SheetHeader>

				<AssetUpsertForm
					key={formKey}
					mode={mode}
					asset={asset}
					defaultDatasetId={defaultDatasetId}
					readOnly={readOnly}
					onSubmit={onSubmit}
					close={() => onOpenChange(false)}
				/>
			</SheetContent>
		</Sheet>
	);
}

function AssetUpsertForm({
	mode,
	asset,
	defaultDatasetId,
	readOnly,
	onSubmit,
	close
}: {
	mode: SheetMode;
	asset: Asset | null;
	defaultDatasetId: string;
	readOnly: boolean;
	onSubmit?: (
		input: AssetUpsertInput,
		ctx: { mode: SheetMode; assetId?: string }
	) => Promise<void> | void;
	close: () => void;
}) {
	const [name, setName] = useState(asset?.name ?? '');
	const [displayName, setDisplayName] = useState(asset?.display_name ?? '');
	const [description, setDescription] = useState(asset?.description ?? '');
	const [datasetId, setDatasetId] = useState(asset?.dataset_id ?? defaultDatasetId);

	const [tags, setTags] = useState<string[]>(asset?.tags ?? []);
	const [tagInput, setTagInput] = useState('');

	function addTag(raw: string) {
		const normalized = normalizeTag(raw);
		if (!normalized) return;

		setTags((prev) => dedupeTokens([...prev, normalized]));
		setTagInput('');
	}

	function removeTag(tag: string) {
		setTags((prev) => prev.filter((t) => t.toLowerCase() !== tag.toLowerCase()));
	}

	async function handleSubmit() {
		const input: AssetUpsertInput = {
			name: name.trim(),
			display_name: displayName.trim(),
			description: description.trim(),
			dataset_id: datasetId.trim(),
			tags
		};

		try {
			await onSubmit?.(input, { mode, assetId: asset?.id });
			toast.success(mode === 'create' ? 'Created' : 'Saved');
			close();
		} catch {
			toast.error(mode === 'create' ? 'Create failed' : 'Update failed');
		}
	}

	return (
		<div className="flex flex-1 flex-col gap-4 overflow-y-auto py-4 text-sm">
			<div className="flex flex-col gap-3">
				<Label htmlFor="asset-name">Name (snake_case)</Label>
				<Input
					id="asset-name"
					value={name}
					onChange={(e) => setName(e.target.value)}
					disabled={readOnly || mode !== 'create'}
					placeholder="e.g. sales_order_lines_daily"
				/>
			</div>

			<div className="flex flex-col gap-3">
				<Label htmlFor="asset-display-name">Display name</Label>
				<Input
					id="asset-display-name"
					value={displayName}
					onChange={(e) => setDisplayName(e.target.value)}
					disabled={readOnly}
					placeholder="e.g. Sales Order Lines Daily"
				/>
			</div>

			<div className="flex flex-col gap-3">
				<Label htmlFor="asset-description">Description</Label>
				<Textarea
					id="asset-description"
					value={description}
					onChange={(e) => setDescription(e.target.value)}
					disabled={readOnly}
					placeholder="Describe this asset..."
					className="min-h-24 resize-y"
				/>
			</div>

			<div className="flex flex-col gap-3">
				<Label htmlFor="asset-dataset">Dataset ID</Label>
				<Input
					id="asset-dataset"
					value={datasetId}
					onChange={(e) => setDatasetId(e.target.value)}
					disabled={readOnly || mode !== 'create'}
					placeholder="Dataset UUID"
				/>
			</div>

			<div className="flex flex-col gap-3">
				<Label htmlFor="asset-tags">Tags</Label>
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
							id="asset-tags"
							value={tagInput}
							onChange={(e) => setTagInput(e.target.value)}
							onKeyDown={(e) => {
								if (e.key === 'Enter' || e.key === ',') {
									e.preventDefault();
									addTag(tagInput);
								}
							}}
							className="placeholder:text-muted-foreground min-w-40 flex-1 border-transparent bg-transparent text-sm outline-none focus:border-transparent focus:ring-0"
							placeholder="Add tag"
							disabled={readOnly}
						/>
					) : null}
				</div>
			</div>

			{asset ? (
				<>
					<Separator />
					<div className="text-muted-foreground grid gap-2 text-xs">
						<div className="flex justify-between gap-3">
							<span>Connector</span>
							<span className="text-foreground">{asset.connector.type}</span>
						</div>
						<div className="flex justify-between gap-3">
							<span>Allowed formats</span>
							<span className="text-foreground">{asset.connector.allowed_formats.join(', ')}</span>
						</div>
						<div className="flex justify-between gap-3">
							<span>Storage prefix</span>
							<span className="text-foreground">{asset.connector.storage_prefix}</span>
						</div>
						<div className="flex justify-between gap-3">
							<span>Created</span>
							<span className="text-foreground">{new Date(asset.created_at).toLocaleString()}</span>
						</div>
						<div className="flex justify-between gap-3">
							<span>Updated</span>
							<span className="text-foreground">{new Date(asset.updated_at).toLocaleString()}</span>
						</div>
					</div>
				</>
			) : null}

			{!readOnly ? (
				<SheetFooter className="mt-auto gap-2">
					<SheetClose asChild>
						<Button variant="outline" type="button">
							Cancel
						</Button>
					</SheetClose>
					<Button type="button" onClick={handleSubmit}>
						{mode === 'create' ? 'Create asset' : 'Save changes'}
					</Button>
				</SheetFooter>
			) : (
				<SheetFooter className="mt-auto gap-2">
					<SheetClose asChild>
						<Button type="button">Close</Button>
					</SheetClose>
				</SheetFooter>
			)}
		</div>
	);
}
