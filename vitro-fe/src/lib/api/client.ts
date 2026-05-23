import z from 'zod';
import { env } from '@/lib/env';
import { ApiErrorBodySchema, VitroApiError } from '@/lib/api/errors';

function joinUrl(baseUrl: string, path: string) {
	const base = baseUrl.endsWith('/') ? baseUrl.slice(0, -1) : baseUrl;
	const p = path.startsWith('/') ? path : `/${path}`;
	return `${base}${p}`;
}

function buildQuery(query?: Record<string, string | number | boolean | null | undefined>) {
	if (!query) return '';
	const params = new URLSearchParams();
	for (const [key, value] of Object.entries(query)) {
		if (value === undefined || value === null) continue;
		params.set(key, String(value));
	}
	const out = params.toString();
	return out ? `?${out}` : '';
}

type ApiFetchJsonBaseArgs<T> = {
	method: 'GET' | 'POST' | 'PATCH' | 'DELETE';
	path: string;
	query?: Record<string, string | number | boolean | null | undefined>;
	body?: unknown;
	schema: z.ZodType<T>;
};

async function apiFetchJsonInternal<T>({
	method,
	path,
	query,
	body,
	headers,
	schema
}: {
	method: 'GET' | 'POST' | 'PATCH' | 'DELETE';
	path: string;
	query?: Record<string, string | number | boolean | null | undefined>;
	body?: unknown;
	headers: Record<string, string>;
	schema: z.ZodType<T>;
}): Promise<T> {
	const url = joinUrl(env.apiBaseUrl, `${path}${buildQuery(query)}`);

	const res = await fetch(url, {
		method,
		headers: {
			...headers,
			...(body === undefined ? {} : { 'Content-Type': 'application/json' })
		},
		body: body === undefined ? undefined : JSON.stringify(body)
	});

	const json = await res.json();

	if (!res.ok) {
		const parsed = ApiErrorBodySchema.parse(json);
		throw new VitroApiError({
			status: res.status,
			code: parsed.error.code,
			message: parsed.error.message,
			correlationId: parsed.error.correlation_id,
			details: parsed.error.details
		});
	}

	return schema.parse(json);
}

export async function apiFetchJsonPublic<T>(args: ApiFetchJsonBaseArgs<T>): Promise<T> {
	return apiFetchJsonInternal({
		...args,
		headers: {}
	});
}

export async function apiFetchJsonBearer<T>(
	args: ApiFetchJsonBaseArgs<T> & { accessToken: string }
): Promise<T> {
	return apiFetchJsonInternal({
		...args,
		headers: { Authorization: `Bearer ${args.accessToken}` }
	});
}

export async function apiFetchJsonWorkspace<T>(
	args: ApiFetchJsonBaseArgs<T> & { accessToken: string; workspaceId: string }
): Promise<T> {
	return apiFetchJsonInternal({
		...args,
		headers: {
			Authorization: `Bearer ${args.accessToken}`,
			'X-Workspace-Id': args.workspaceId
		}
	});
}
