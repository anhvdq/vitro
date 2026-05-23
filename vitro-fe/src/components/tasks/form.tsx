import { toast } from 'sonner';
import { useMemo, useState } from 'react';

import { Task, TaskConfigs, TaskConfigsSchema } from '@/lib/tasks/types';
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

type SheetMode = 'create' | 'view';

export type TaskUpsertInput = {
	asset_id: string;
	task_type: 'FULL_ANALYSIS';
	configs: TaskConfigs;
};

const defaultConfigs: TaskConfigs = {
	mode: 'AUTO',
	profiling: { sampling: 'SAMPLE', sample_rows: 500000, seed: 42 },
	generate: ['DESCRIPTION', 'SCHEMA', 'PROFILE', 'DQ_BASELINE', 'FILE_PATTERN', 'GRAIN_HINTS']
};

export function TaskUpsertSheet({
	open,
	onOpenChange,
	mode,
	task,
	onSubmit
}: {
	open: boolean;
	onOpenChange: (v: boolean) => void;
	mode: SheetMode;
	task: Task | null;
	onSubmit?: (input: TaskUpsertInput) => Promise<void> | void;
}) {
	const readOnly = mode === 'view';
	const formKey = `${mode}:${task?.id ?? 'new'}`;

	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent side="right" className="flex flex-col">
				<SheetHeader className="gap-1">
					<SheetTitle>{mode === 'create' ? 'Create task' : 'Task details'}</SheetTitle>
					<SheetDescription>
						{mode === 'create' ? 'Create a task in this workspace.' : 'View task information.'}
					</SheetDescription>
				</SheetHeader>

				<TaskUpsertForm
					key={formKey}
					mode={mode}
					task={task}
					readOnly={readOnly}
					onSubmit={onSubmit}
					close={() => onOpenChange(false)}
				/>
			</SheetContent>
		</Sheet>
	);
}

function TaskUpsertForm({
	mode,
	task,
	readOnly,
	onSubmit,
	close
}: {
	mode: SheetMode;
	task: Task | null;
	readOnly: boolean;
	onSubmit?: (input: TaskUpsertInput) => Promise<void> | void;
	close: () => void;
}) {
	const [assetId, setAssetId] = useState(task?.asset_id ?? '');

	const initialConfigs = useMemo(() => {
		return JSON.stringify(task?.configs ?? defaultConfigs, null, 2);
	}, [task?.configs]);

	const [configsText, setConfigsText] = useState(initialConfigs);

	function parseConfigs(raw: string) {
		return TaskConfigsSchema.parse(JSON.parse(raw));
	}

	async function handleSubmit() {
		let parsedConfigs: TaskConfigs;
		try {
			parsedConfigs = parseConfigs(configsText);
		} catch {
			toast.error('Configs must be valid JSON');
			return;
		}

		const input: TaskUpsertInput = {
			asset_id: assetId.trim(),
			task_type: 'FULL_ANALYSIS',
			configs: parsedConfigs
		};

		try {
			await onSubmit?.(input);
			toast.success('Created');
			close();
		} catch {
			toast.error('Create failed');
		}
	}

	return (
		<div className="flex flex-1 flex-col gap-4 overflow-y-auto py-4 text-sm">
			<div className="flex flex-col gap-3">
				<Label htmlFor="task-type">Task Type</Label>
				<Input id="task-type" value="FULL_ANALYSIS" disabled />
			</div>

			<div className="flex flex-col gap-3">
				<Label htmlFor="task-asset">Asset ID</Label>
				<Input
					id="task-asset"
					value={assetId}
					onChange={(e) => setAssetId(e.target.value)}
					disabled={readOnly}
					placeholder="Asset UUID"
				/>
			</div>

			<div className="flex flex-col gap-3">
				<Label htmlFor="task-configs">Configs (JSON)</Label>
				<Textarea
					id="task-configs"
					value={configsText}
					onChange={(e) => setConfigsText(e.target.value)}
					disabled={readOnly}
					className="min-h-40 font-mono text-xs"
					placeholder="{}"
				/>
			</div>

			{mode === 'create' ? (
				<SheetFooter className="mt-auto gap-2">
					<SheetClose asChild>
						<Button variant="outline" type="button">
							Cancel
						</Button>
					</SheetClose>
					<Button type="button" onClick={handleSubmit}>
						Create task
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
