import { describe, expect, it, vi } from 'vitest';
import z from 'zod';
import { VitroApiError } from '@/lib/api/errors';

vi.mock('@/lib/env', () => ({
	env: {
		apiBaseUrl: 'https://api.test'
	}
}));

import { apiFetchJsonWorkspace } from '@/lib/api/client';

describe('apiFetchJsonWorkspace', () => {
	it('adds auth and workspace headers', async () => {
		const fetchMock = vi.fn().mockResolvedValue({
			ok: true,
			status: 200,
			json: async () => ({ data: 'ok' })
		});
		globalThis.fetch = fetchMock as any;

		const res = await apiFetchJsonWorkspace({
			method: 'GET',
			path: '/datasets',
			accessToken: 'token',
			workspaceId: 'ws',
			schema: z.object({ data: z.string() })
		});

		expect(res.data).toBe('ok');
		expect(fetchMock).toHaveBeenCalledWith('https://api.test/datasets', {
			method: 'GET',
			headers: { Authorization: 'Bearer token', 'X-Workspace-Id': 'ws' },
			body: undefined
		});
	});

	it('throws VitroApiError on non-ok responses', async () => {
		const fetchMock = vi.fn().mockResolvedValue({
			ok: false,
			status: 400,
			json: async () => ({
				error: {
					code: 'VALIDATION_ERROR',
					message: 'name is required',
					details: { field: 'name' },
					correlation_id: 'corr-1'
				}
			})
		});
		globalThis.fetch = fetchMock as any;

		await expect(
			apiFetchJsonWorkspace({
				method: 'POST',
				path: '/datasets',
				accessToken: 'token',
				workspaceId: 'ws',
				body: { name: '' },
				schema: z.object({ data: z.unknown() })
			})
		).rejects.toEqual(
			expect.objectContaining({
				name: 'VitroApiError',
				status: 400,
				code: 'VALIDATION_ERROR',
				correlationId: 'corr-1'
			} satisfies Partial<VitroApiError>)
		);
	});
});
