// Caminho relativo: app/web/src/lib/locale.ts
// First-visit locale choice (IDI-03): any Portuguese preference -> pt-br, anything else -> en.
// Later visits use the choice stored by Paraglide's localStorage strategy (IDI-04).

export type AppLocale = 'pt-br' | 'en';

export function detectLocale(preferred: readonly string[]): AppLocale {
	const first = preferred.find((tag) => tag.trim() !== '');
	if (first === undefined) return 'en';
	return first.toLowerCase().split('-')[0] === 'pt' ? 'pt-br' : 'en';
}

/** Paraglide stores the user's choice under this key (strategy "localStorage"). */
export const LOCALE_STORAGE_KEY = 'PARAGLIDE_LOCALE';

/** True when the user already picked a language in an earlier visit. */
export function hasStoredLocale(storage: Pick<Storage, 'getItem'> | undefined): boolean {
	try {
		return storage?.getItem(LOCALE_STORAGE_KEY) != null;
	} catch {
		return false;
	}
}
