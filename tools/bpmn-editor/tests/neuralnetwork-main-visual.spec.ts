import { expect, test } from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

test('NeuralNetwork main.bpmn renders gateways and yes/no labels', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1600, height: 1000 });

  const here = path.dirname(fileURLToPath(import.meta.url));
  const repoRoot = path.resolve(here, '..', '..', '..');
  const bpmnPath = path.join(
    repoRoot,
    'conversion',
    'Examples',
    'NeuralNetwork',
    'src',
    'main.bpmn'
  );

  const xml = fs.readFileSync(bpmnPath, 'utf8');
  expect(xml).toMatch(/\bsequenceFlow\b[^>]*\bname="yes"/);
  expect(xml).toMatch(/\bsequenceFlow\b[^>]*\bname="no"/);

  await page.setContent(`<!doctype html>
    <html>
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>Visual Test</title>
        <style>
          html, body { margin: 0; padding: 0; height: 100%; }
          #canvas { width: 100vw; height: 100vh; background: white; }
        </style>
      </head>
      <body>
        <div id="canvas"></div>
        <script type="module">
          import Modeler from 'https://esm.sh/bpmn-js@18.5.0/lib/Modeler?bundle';
          import { layoutProcess } from 'https://esm.sh/bpmn-auto-layout@1.1.1?bundle';

          const xml = ${JSON.stringify(xml)};
          const canvas = document.getElementById('canvas');

          function hasBpmnDi(s) {
            return /<\w*:?BPMNDiagram\b|<\w*:?BPMNPlane\b|xmlns:bpmndi=/.test(s);
          }

          const modeler = new Modeler({ container: canvas });
          const xmlToImport = hasBpmnDi(xml) ? xml : await layoutProcess(xml);
          await modeler.importXML(xmlToImport);
          modeler.get('canvas').zoom('fit-viewport');
          window.__READY__ = true;
        </script>
      </body>
    </html>`);

  await page.waitForFunction(() => (window as any).__READY__ === true, null, { timeout: 20_000 });

  const gatewayCount = await page
    .locator('g.djs-element[data-element-id^="ExclusiveGateway_"]')
    .count();
  expect(gatewayCount).toBeGreaterThan(0);

  // Screenshot the diagram container for visual inspection/regression.
  await page.locator('#canvas').screenshot({ path: testInfo.outputPath('neuralnetwork-main.png') });
});
