import { UploadIcon } from 'lucide-react';
import { useRef, useState } from 'react';
import { toast } from 'sonner';

import { useUploadAssetFilesMutation } from '@/lib/files/queries';

function errorMessage(err: unknown): string {
	return err instanceof Error ? err.message : 'Upload failed';
}

export function FileDropZone({ assetId }: { assetId: string }) {
	const [isDragging, setIsDragging] = useState(false);
	const inputRef = useRef<HTMLInputElement>(null);
	const uploadMutation = useUploadAssetFilesMutation();

	const uploadFiles = (files: File[]) => {
		if (!files.length) return;

		uploadMutation.mutate(
			{ assetId, files },
			{
				onSuccess: (results) => {
					const succeeded = results.filter((r) => r.status === 'success').length;
					const failed = results.length - succeeded;

					if (failed === 0) {
						toast.success(`Uploaded ${succeeded} file${succeeded === 1 ? '' : 's'}`);
					} else if (succeeded === 0) {
						if (failed === 1) {
							toast.error(errorMessage(results[0].error));
						} else {
							toast.error(`Failed to upload ${failed} file${failed === 1 ? '' : 's'}`);
						}
					} else {
						toast.warning(`Uploaded ${succeeded}, failed ${failed}`);
					}
				},
				onError: (err: unknown) => {
					toast.error(err instanceof Error ? err.message : 'Upload failed');
				}
			}
		);
	};

	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: drag-and-drop handlers only (no click/keyboard interaction here - the "browse" button is the sole keyboard-accessible entry point).
		<div
			className={[
				'flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed p-6 text-center transition-colors',
				isDragging ? 'border-primary bg-muted/50' : 'border-muted-foreground/25'
			].join(' ')}
			onDragOver={(e) => {
				e.preventDefault();
				setIsDragging(true);
			}}
			onDragLeave={(e) => {
				e.preventDefault();
				setIsDragging(false);
			}}
			onDrop={(e) => {
				e.preventDefault();
				setIsDragging(false);
				uploadFiles(Array.from(e.dataTransfer.files));
			}}
		>
			<UploadIcon className="text-muted-foreground h-6 w-6" />
			<p className="text-sm">
				{uploadMutation.isPending ? (
					`Uploading ${uploadMutation.variables?.files.length ?? 0} file${
						(uploadMutation.variables?.files.length ?? 0) === 1 ? '' : 's'
					}…`
				) : (
					<>
						Drag and drop files here, or{' '}
						<button
							type="button"
							className="text-primary underline underline-offset-2"
							onClick={() => inputRef.current?.click()}
						>
							browse
						</button>
					</>
				)}
			</p>
			<input
				ref={inputRef}
				type="file"
				multiple
				accept=".csv,text/csv"
				className="hidden"
				onChange={(e) => {
					uploadFiles(Array.from(e.target.files ?? []));
					e.target.value = '';
				}}
			/>
		</div>
	);
}
