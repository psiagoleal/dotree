// Caminho relativo: app/web/src/lib/download.ts
// Saves text or bytes the page generated as a file, without any server (GER-05).

export function download(
	filename: string,
	data: string | Uint8Array<ArrayBuffer>,
	type: string
): void {
	const url = URL.createObjectURL(new Blob([data], { type }));
	const a = document.createElement('a');
	a.href = url;
	a.download = filename;
	document.body.append(a);
	a.click();
	a.remove();
	// Some browsers read the URL after click returns.
	setTimeout(() => URL.revokeObjectURL(url), 1000);
}
