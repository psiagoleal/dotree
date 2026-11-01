// Caminho relativo: app/web/src/lib/view.ts
// Dots or indexed variables (VIS-01): the choice is remembered across visits and toggled
// with the V key (VIS-03, `keys.ts`). Storage may be missing or blocked; the default is then "dots".

export type ViewMode = 'dots' | 'indexed';
export const VIEW_STORAGE_KEY = 'dotree.view';

export function readView(storage: Pick<Storage, 'getItem'> | undefined): ViewMode {
	try {
		return storage?.getItem(VIEW_STORAGE_KEY) === 'indexed' ? 'indexed' : 'dots';
	} catch {
		return 'dots';
	}
}

export function writeView(storage: Pick<Storage, 'setItem'> | undefined, mode: ViewMode): void {
	try {
		storage?.setItem(VIEW_STORAGE_KEY, mode);
	} catch {
		// A blocked storage only costs the memory of the choice.
	}
}

/** The browser's localStorage, or undefined where touching it throws (blocked site data). */
export function browserStorage(): Storage | undefined {
	try {
		return globalThis.localStorage;
	} catch {
		return undefined;
	}
}
