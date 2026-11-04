// Caminho relativo: app/web/src/lib/download.ts
// Saves text the page generated as a file, without any server (GER-05).

export function download(filename: string, text: string, type: string): void {
	const url = URL.createObjectURL(new Blob([text], { type }));
	const a = document.createElement('a');
	a.href = url;
	a.download = filename;
	document.body.append(a);
	a.click();
	a.remove();
	// Some browsers read the URL after click returns.
	setTimeout(() => URL.revokeObjectURL(url), 1000);
}
