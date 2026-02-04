import { expect, test } from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

function repoRootFromHere(here: string): string {
  return path.resolve(here, '..', '..', '..');
}

test('tabs + open/save are visually stable', async ({ page }, testInfo) => {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const repoRoot = repoRootFromHere(here);
  const openPath = path
    .join(repoRoot, 'conversion', 'Examples', 'NeuralNetwork', 'src', 'main.bpmn')
    .replaceAll('/', '\\');
  const savePath = path.join(repoRoot, 'target', 'tmp', 'playwright-saved.bpmn').replaceAll('/', '\\');

  fs.mkdirSync(path.dirname(savePath), { recursive: true });
  if (fs.existsSync(savePath)) {
    fs.unlinkSync(savePath);
  }

  const visualMode = (process.env.PW_VISUAL_MODE ?? 'compare').toLowerCase();
  const compareScreenshots = visualMode !== 'debug';
  async function snap(name: string) {
    const body = page.locator('body');
    if (compareScreenshots) {
      await expect(body).toHaveScreenshot(name);
    } else {
      await body.screenshot({ path: testInfo.outputPath(name) });
    }
  }

  await page.addInitScript(
    ({ openPath, savePath }) => {
      (window as any).__RUST_TO_TS_E2E__ = {
        dialog: {
          openQueue: [openPath],
          saveQueue: [savePath]
        }
      };
    },
    { openPath, savePath }
  );

  await page.goto('/');

  // Open the BPMN file (uses e2e dialog + virtual fs)
  await page.click('#openNativeBtn');
  await expect(page.locator('#status')).toContainText('Loaded', { timeout: 20_000 });

  // BPMN tab
  await page.click('#tabBpmn');
  await expect(page.locator('#bpmnEditor')).toBeVisible();
  await expect(page.locator('#canvas svg[data-element-id="Process_1"]').first()).toBeVisible();
  await snap('tab-bpmn.png');

  // Convert to Rust + Rust tab
  await page.click('#convertRustBtn');
  await expect(page.locator('#status')).toContainText('Converted to Rust', { timeout: 20_000 });
  await page.click('#tabRust');
  await expect(page.locator('#codeEditor')).toBeVisible();
  await snap('tab-rust.png');

  // Convert to TS + TS tab
  await page.click('#convertTsBtn');
  await expect(page.locator('#status')).toContainText('Converted to TS', { timeout: 20_000 });
  await page.click('#tabTs');
  await expect(page.locator('#codeEditor')).toBeVisible();
  await snap('tab-ts.png');

  // Validate -> dismiss modal -> Validate tab snapshot
  await page.click('#validateBtn');
  // modal shows after validate; close it.
  await expect(page.locator('#modalOverlay')).toBeVisible({ timeout: 20_000 });
  await page.click('#modalCloseBtn');

  await page.click('#tabOut');
  await expect(page.locator('#outPanel')).toBeVisible();
  await expect(page.locator('#outPanel')).toContainText('OK: true');
  await snap('tab-validate.png');

  // Save As
  await page.click('#saveAsNativeBtn');
  await expect(page.locator('#status')).toContainText('Saved', { timeout: 20_000 });
  await snap('after-saveas.png');
});
