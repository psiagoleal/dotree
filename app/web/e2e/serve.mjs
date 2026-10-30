// Caminho relativo: app/web/e2e/serve.mjs
// Minimal static server that mimics GitHub Pages: serves <dir> under <base>/ and answers
// 404.html for unknown paths. Used only by the e2e suite (ENT-03).
// Usage: node e2e/serve.mjs <dir> <port> <base>
import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { extname, join, normalize, resolve } from 'node:path';

const [dir = 'build', port = '4173', base = ''] = process.argv.slice(2);
const root = resolve(dir);
const types = {
	'.html': 'text/html; charset=utf-8',
	'.js': 'text/javascript',
	'.css': 'text/css',
	'.json': 'application/json',
	'.svg': 'image/svg+xml',
	'.png': 'image/png',
	'.wasm': 'application/wasm',
	'.txt': 'text/plain'
};

async function fileFor(pathname) {
	const rel = normalize(decodeURIComponent(pathname)).replace(/^([/\\])+/, '');
	const candidate = join(root, rel);
	if (!candidate.startsWith(root)) return null;
	for (const p of [candidate, join(candidate, 'index.html'), `${candidate}.html`]) {
		const s = await stat(p).catch(() => null);
		if (s?.isFile()) return p;
	}
	return null;
}

createServer(async (req, res) => {
	const url = new URL(req.url ?? '/', 'http://localhost');
	if (!url.pathname.startsWith(`${base}/`) && url.pathname !== base) {
		res.writeHead(404).end('outside base path');
		return;
	}
	const file = await fileFor(url.pathname.slice(base.length) || '/');
	const target = file ?? join(root, '404.html');
	const body = await readFile(target).catch(() => null);
	if (!body) {
		res.writeHead(404).end('not found');
		return;
	}
	res.writeHead(file ? 200 : 404, {
		'content-type': types[extname(target)] ?? 'application/octet-stream'
	});
	res.end(body);
}).listen(Number(port), () => console.log(`serving ${root} at http://localhost:${port}${base}/`));
