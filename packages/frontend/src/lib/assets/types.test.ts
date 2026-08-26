import { describe, expect, it } from 'vitest';
import { AssetSchema } from '@/lib/assets/types';

const validAsset = {
	id: 'a1',
	workspace_id: 'ws-1',
	dataset_id: 'd1',
	name: 'sales_data',
	display_name: 'Sales Data',
	description: 'desc',
	owner_user_id: 'u1',
	owner_user_name: 'User',
	tags: ['sales'],
	connector: {
		id: 'c1',
		type: 'UPLOAD',
		storage_prefix: 'ws/ws-1/assets/a1',
		allowed_formats: ['CSV']
	},
	status: 'ACTIVE' as const,
	created_at: '2025-01-01T00:00:00Z',
	updated_at: '2025-01-01T00:00:00Z',
	archived_at: null
};

describe('AssetSchema', () => {
	it('parses payload with workspace_id', () => {
		const parsed = AssetSchema.parse(validAsset);
		expect(parsed.workspace_id).toBe('ws-1');
	});

	it('requires workspace_id', () => {
		const withoutWorkspace = { ...validAsset };
		delete (withoutWorkspace as { workspace_id?: string }).workspace_id;

		expect(() => AssetSchema.parse(withoutWorkspace)).toThrow();
	});
});
