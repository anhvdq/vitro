import en from '../../../contracts/i18n/en.json';
import vi from '../../../contracts/i18n/vi.json';

export type MessageKey = keyof typeof vi;
type Locale = 'vi' | 'en';

const catalogs: Record<Locale, Record<MessageKey, string>> = { vi, en };

/** Looks up `key` in the catalog and fills `{name}` placeholders. */
export function createTranslator(locale: Locale) {
	return (key: MessageKey, params: Record<string, string> = {}): string =>
		catalogs[locale][key].replace(/\{(\w+)\}/g, (_, name: string) => params[name] ?? `{${name}}`);
}

/** Vietnamese is the default locale. */
export const t = createTranslator('vi');
