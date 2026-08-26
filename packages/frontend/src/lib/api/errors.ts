import z from 'zod';

export const ApiErrorBodySchema = z.object({
	error: z.object({
		code: z.string(),
		message: z.string(),
		details: z.unknown().optional(),
		correlation_id: z.string()
	})
});

export type ApiErrorBody = z.infer<typeof ApiErrorBodySchema>;

export class VitroApiError extends Error {
	readonly status: number;
	readonly code: string;
	readonly correlationId: string;
	readonly details?: unknown;

	constructor(args: {
		status: number;
		code: string;
		message: string;
		correlationId: string;
		details?: unknown;
	}) {
		super(args.message);
		this.name = 'VitroApiError';
		this.status = args.status;
		this.code = args.code;
		this.correlationId = args.correlationId;
		this.details = args.details;
	}
}
