import { defineConfig, devices } from '@playwright/test';

const slowMo = (() => {
  const raw = process.env.PW_SLOWMO;
  if (!raw) return 0;
  const n = Number(raw);
  return Number.isFinite(n) && n > 0 ? n : 0;
})();

export default defineConfig({
  testDir: './tests',
  snapshotDir: './tests/__snapshots__',
  timeout: 60_000,
  expect: {
    timeout: 15_000,
    toHaveScreenshot: {
      maxDiffPixelRatio: 0,
      animations: 'disabled'
    }
  },
  retries: 0,
  reporter: [['list'] as any],
  use: {
    ...devices['Desktop Edge'],
    viewport: { width: 1600, height: 900 },
    deviceScaleFactor: 1,
    locale: 'en-US',
    timezoneId: 'UTC',
    colorScheme: 'dark',
    baseURL: 'http://127.0.0.1:15000',
    launchOptions: slowMo ? { slowMo } : undefined,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure'
  },
  projects: [
    {
      name: 'msedge',
      use: {
        browserName: 'chromium',
        channel: 'msedge'
      }
    }
  ],
  webServer: [
    {
      command: 'cmd /c "cd ..\\.. && cargo run --bin bpmn-editor-backend -- --port 15123"',
      url: 'http://127.0.0.1:15123/health',
      reuseExistingServer: true,
      timeout: 120_000
    },
    {
      command:
        'cmd /c "set VITE_RUST_BACKEND_URL=http://127.0.0.1:15123&& npm run vite:dev -- --host 127.0.0.1 --port 15000"',
      url: 'http://127.0.0.1:15000',
      reuseExistingServer: false,
      timeout: 60_000
    }
  ]
});
