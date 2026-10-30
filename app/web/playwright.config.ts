// Caminho relativo: app/web/playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

// The e2e suite runs against the production build, under the same base path that GitHub
// Pages uses (ENT-03). `pnpm run build` must have been run with BASE_PATH=/dotree first.
const base = process.env.BASE_PATH ?? '/dotree';
const port = Number(process.env.E2E_PORT ?? 4180);

export default defineConfig({
	testDir: 'e2e',
	testMatch: '**/*.e2e.ts',
	forbidOnly: !!process.env.CI,
	reporter: process.env.CI ? 'github' : 'list',
	use: { baseURL: `http://localhost:${port}${base}/` },
	projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
	webServer: {
		command: `node e2e/serve.mjs build ${port} ${base}`,
		port,
		reuseExistingServer: !process.env.CI
	}
});
