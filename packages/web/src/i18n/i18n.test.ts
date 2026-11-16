import { describe, expect, it } from 'vitest';
import en from '../../../contracts/i18n/en.json';
import vi from '../../../contracts/i18n/vi.json';
import { createTranslator } from './index';

const placeholders = (message: string) =>
	[...message.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

describe('message catalogs', () => {
	it('define the same keys in Vietnamese and English', () => {
		// A key missing in one language would show users a raw key or a blank.
		expect(Object.keys(en).sort()).toEqual(Object.keys(vi).sort());
	});

	it('use the same placeholders in both languages', () => {
		for (const [key, message] of Object.entries(vi)) {
			expect(placeholders(en[key as keyof typeof en]), key).toEqual(placeholders(message));
		}
	});

	it('fills placeholders and leaves unknown ones visible', () => {
		const t = createTranslator('en');
		expect(t('home.signedInAs', { email: 'a@b.c' })).toBe('Signed in as a@b.c');
		expect(t('home.signedInAs')).toBe('Signed in as {email}');
	});
});
