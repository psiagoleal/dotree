// Caminho relativo: app/web/src/lib/persist.ts
// The exercise in progress, kept in local storage so closing the tab loses nothing (SAL-01).
// Storage may be missing, blocked or full; every function here reports that instead of
// throwing, and the page warns once (SAL-04).
import type { CompressionSession } from '#lib/core.ts';

export const EXERCISE_STORAGE_KEY = 'dotree.exercise';

export type Saved = { kind: 'none' } | { kind: 'unavailable' } | { kind: 'found'; text: string };

export function loadSaved(storage: Pick<Storage, 'getItem'> | undefined): Saved {
	if (!storage) return { kind: 'unavailable' };
	try {
		const text = storage.getItem(EXERCISE_STORAGE_KEY);
		return text === null ? { kind: 'none' } : { kind: 'found', text };
	} catch {
		return { kind: 'unavailable' };
	}
}

/** False when the text could not be stored. */
export function saveExercise(storage: Pick<Storage, 'setItem'> | undefined, text: string): boolean {
	if (!storage) return false;
	try {
		storage.setItem(EXERCISE_STORAGE_KEY, text);
		return true;
	} catch {
		return false;
	}
}

export function clearSaved(storage: Pick<Storage, 'removeItem'> | undefined): void {
	try {
		storage?.removeItem(EXERCISE_STORAGE_KEY);
	} catch {
		// Nothing to clear where nothing could be stored.
	}
}

/** Calls that change the exercise or reveal something (and so change the file). */
const CHANGES = new Set<string>([
	'formCell',
	'concludeLevel',
	'closeWithCpa',
	'goToLevel',
	'undo',
	'redo',
	'hint',
	'nextMove',
	'lowerBound',
	'solution',
	'applySolution',
	'revealWidth',
	'revealWidthPath'
]);

/** The same session, calling `save` with the exercise file after every change. */
export function autosaving(
	session: CompressionSession,
	save: (text: string) => void
): CompressionSession {
	return new Proxy(session, {
		get(target, key) {
			const value: unknown = Reflect.get(target, key);
			if (typeof key !== 'string' || !CHANGES.has(key) || typeof value !== 'function') {
				return value;
			}
			return (...args: unknown[]) => {
				const result: unknown = value.apply(target, args);
				save(target.exercise());
				return result;
			};
		}
	});
}
