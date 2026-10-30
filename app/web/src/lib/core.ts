// Caminho relativo: app/web/src/lib/core.ts
// Loads the dotree-core WebAssembly module. The interface never reimplements domain rules
// (ADR 0001); it only calls into this module.
import init, { coreVersion } from '#lib/wasm/dotree.js';

export interface Core {
	version: string;
}

export async function loadCore(): Promise<Core> {
	await init();
	return { version: coreVersion() };
}
