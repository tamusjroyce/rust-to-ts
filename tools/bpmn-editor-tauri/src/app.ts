import Modeler from "bpmn-js/lib/Modeler";
import { invoke, open, save } from "./tauri_shim";

import * as monaco from "monaco-editor";
import "monaco-editor/esm/vs/basic-languages/rust/rust.contribution";

const canvas = document.getElementById("canvas") as HTMLDivElement;
const status = document.getElementById("status") as HTMLDivElement;

const newBtn = document.getElementById("newBtn") as HTMLButtonElement;
const openNativeBtn = document.getElementById("openNativeBtn") as HTMLButtonElement;
const saveNativeBtn = document.getElementById("saveNativeBtn") as HTMLButtonElement;
const saveAsNativeBtn = document.getElementById("saveAsNativeBtn") as HTMLButtonElement;
const convertRustBtn = document.getElementById("convertRustBtn") as HTMLButtonElement;
const convertTsBtn = document.getElementById("convertTsBtn") as HTMLButtonElement;
const syncFromRustBtn = document.getElementById("syncFromRustBtn") as HTMLButtonElement;
const syncFromTsBtn = document.getElementById("syncFromTsBtn") as HTMLButtonElement;
const validateBtn = document.getElementById("validateBtn") as HTMLButtonElement;

const tabBpmn = document.getElementById("tabBpmn") as HTMLButtonElement;
const tabRust = document.getElementById("tabRust") as HTMLButtonElement;
const tabTs = document.getElementById("tabTs") as HTMLButtonElement;
const tabOut = document.getElementById("tabOut") as HTMLButtonElement;

const applyXmlBtn = document.getElementById("applyXmlBtn") as HTMLButtonElement;
const syncFromDiagramBtn = document.getElementById(
  "syncFromDiagramBtn"
) as HTMLButtonElement;

const bpmnEditor = document.getElementById("bpmnEditor") as HTMLTextAreaElement;
const codeEditorHost = document.getElementById("codeEditor") as HTMLDivElement;
const outPanel = document.getElementById("outPanel") as HTMLPreElement;

const modalOverlay = document.getElementById("modalOverlay") as HTMLDivElement;
const modalTitle = document.getElementById("modalTitle") as HTMLDivElement;
const modalBody = document.getElementById("modalBody") as HTMLDivElement;
const modalActions = document.getElementById("modalActions") as HTMLDivElement;
const modalCloseBtn = document.getElementById("modalCloseBtn") as HTMLButtonElement;

type StatusTone = "neutral" | "ok" | "warn" | "err";

function setStatus(msg: string, tone: StatusTone = "neutral") {
  status.textContent = msg;
  status.classList.toggle("status-ok", tone === "ok");
  status.classList.toggle("status-warn", tone === "warn");
  status.classList.toggle("status-err", tone === "err");
}

function toErrorString(e: unknown): string {
  if (e instanceof Error) return e.message;
  try {
    return JSON.stringify(e);
  } catch {
    return String(e);
  }
}

function showStartupError(title: string, details: string) {
  try {
    const pre = document.createElement("pre");
    pre.textContent = details;
    void showChoiceModal({
      title,
      body: pre,
      choices: [{ id: "close", label: "Close", variant: "primary" }],
      dismissId: "close"
    });
  } catch {
    // If modal wiring fails, at least show status.
    setStatus(`${title}: ${details}`, "err");
  }
}

type ValidateResult = {
  ok: boolean;
  stdout_direct: string;
  stdout_roundtrip: string;
  rust_direct: string;
  rust_roundtrip: string;
  bpmn_roundtrip: string;
};

let activeTab: "bpmn" | "rust" | "ts" | "out" = "bpmn";
let currentXml: string = "";
let editorXml: string = "";
let editorDirty = false;
let currentFilePath: string | null = null;
let lastDir: string | null = null;
let lastSavedXml: string = "";
let lastKnownDiskHash: string | null = null;
let lastRust: string = "";
let lastTs: string = "";
let lastValidate: ValidateResult | null = null;

// A baseline representation of the default template as bpmn-js exports it.
let defaultTemplateCanonical: string | null = null;

const LS_LAST_DIR = "bpmnEditor.lastDir";

function normalizeXmlForCompare(xml: string): string {
  return xml.replace(/\s+/g, " ").trim();
}

function hashStringFNV1a(text: string): string {
  let hash = 0x811c9dc5;
  for (let i = 0; i < text.length; i++) {
    hash ^= text.charCodeAt(i);
    hash = (hash * 0x01000193) >>> 0;
  }
  return hash.toString(16).padStart(8, "0");
}

function pathDirname(p: string): string {
  const normalized = p.replace(/[\\/]+/g, "\\");
  const idx = normalized.lastIndexOf("\\");
  if (idx <= 0) return normalized;
  return normalized.slice(0, idx);
}

function loadLastDirFromStorage() {
  try {
    const v = localStorage.getItem(LS_LAST_DIR);
    if (v && v.trim()) lastDir = v;
  } catch {
    // ignore
  }
}

function saveLastDirToStorage(dir: string) {
  lastDir = dir;
  try {
    localStorage.setItem(LS_LAST_DIR, dir);
  } catch {
    // ignore
  }
}

function setCurrentFilePath(p: string | null) {
  currentFilePath = p;
  if (p) saveLastDirToStorage(pathDirname(p));
  updateButtonStates();
}

function updateButtonStates() {
  saveNativeBtn.disabled = !currentFilePath;
}

type ModalChoice = {
  id: string;
  label: string;
  variant?: "primary" | "danger" | "default";
};

function showChoiceModal(opts: {
  title: string;
  body: string | HTMLElement;
  choices: ModalChoice[];
  dismissId?: string;
}): Promise<string> {
  modalTitle.textContent = opts.title;
  modalBody.innerHTML = "";
  if (typeof opts.body === "string") {
    const p = document.createElement("div");
    p.textContent = opts.body;
    modalBody.appendChild(p);
  } else {
    modalBody.appendChild(opts.body);
  }
  modalActions.innerHTML = "";
  modalOverlay.classList.remove("hidden");

  return new Promise(resolve => {
    const dismissId = opts.dismissId ?? "cancel";

    const cleanup = (id: string) => {
      modalOverlay.classList.add("hidden");
      modalActions.innerHTML = "";
      modalCloseBtn.onclick = null;
      modalOverlay.removeEventListener("click", onOverlayClick);
      window.removeEventListener("keydown", onKeyDown);
      resolve(id);
    };

    const onOverlayClick = (e: MouseEvent) => {
      if (e.target === modalOverlay) cleanup(dismissId);
    };

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") cleanup(dismissId);
    };

    modalOverlay.addEventListener("click", onOverlayClick);
    window.addEventListener("keydown", onKeyDown);
    modalCloseBtn.onclick = () => cleanup(dismissId);

    for (const c of opts.choices) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.textContent = c.label;
      if (c.variant === "primary") btn.classList.add("btnPrimary");
      if (c.variant === "danger") btn.classList.add("btnDanger");
      btn.addEventListener("click", () => cleanup(c.id));
      modalActions.appendChild(btn);
    }
  });
}

function setActiveTab(next: typeof activeTab) {
  activeTab = next;
  for (const [key, el] of [
    ["bpmn", tabBpmn],
    ["rust", tabRust],
    ["ts", tabTs],
    ["out", tabOut]
  ] as const) {
    el.classList.toggle("active", key === activeTab);
  }
  renderPanel();
}

function renderPanel() {
  if (activeTab === "bpmn") {
    bpmnEditor.style.display = "block";
    codeEditorHost.style.display = "none";
    outPanel.style.display = "none";
    bpmnEditor.value = editorXml || currentXml || "";
    return;
  }

  bpmnEditor.style.display = "none";
  outPanel.style.display = "none";
  codeEditorHost.style.display = "block";

  if (activeTab === "rust") {
    codeEditor.setModel(rustModel);
    codeEditor.updateOptions({ readOnly: false });
    if (!rustModel.getValue() && lastRust) rustModel.setValue(lastRust);
    return;
  }
  if (activeTab === "ts") {
    codeEditor.setModel(tsModel);
    codeEditor.updateOptions({ readOnly: false });
    if (!tsModel.getValue() && lastTs) tsModel.setValue(lastTs);
    return;
  }

  // out
  codeEditorHost.style.display = "none";
  outPanel.style.display = "block";
  if (!lastValidate) {
    outPanel.textContent = "(validate to see results)";
    return;
  }
  const v = lastValidate;
  outPanel.textContent = [
    `OK: ${v.ok}`,
    "",
    "--- stdout (direct) ---",
    v.stdout_direct,
    "--- stdout (roundtrip) ---",
    v.stdout_roundtrip,
    "--- bpmn (roundtrip) ---",
    v.bpmn_roundtrip
  ].join("\n");
}

const rustModel = monaco.editor.createModel("", "rust");
const tsModel = monaco.editor.createModel("", "typescript");

const codeEditor = monaco.editor.create(codeEditorHost, {
  theme: "vs-dark",
  automaticLayout: true,
  minimap: { enabled: false },
  fontSize: 12,
  wordWrap: "on",
  tabSize: 2,
  insertSpaces: true,
  readOnly: false,
});

// Stabilize visual output for screenshot tests.
if ((globalThis as any).__RUST_TO_TS_E2E__) {
  codeEditor.updateOptions({
    cursorBlinking: "solid",
    renderLineHighlight: "none",
    renderWhitespace: "none",
    occurrencesHighlight: "off",
    selectionHighlight: false,
    overviewRulerLanes: 0,
  } as any);
}

codeEditor.onDidChangeModelContent(() => {
  const model = codeEditor.getModel();
  if (!model) return;
  const text = model.getValue();
  if (model === rustModel) lastRust = text;
  if (model === tsModel) lastTs = text;
});

const DEFAULT_XML = `<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI"
  xmlns:dc="http://www.omg.org/spec/DD/20100524/DC"
  xmlns:di="http://www.omg.org/spec/DD/20100524/DI"
  id="Definitions_1"
  targetNamespace="http://bpmn.io/schema/bpmn">
  <bpmn:process id="Process_1" isExecutable="false">
    <bpmn:startEvent id="StartEvent_1" name="Start" />
  </bpmn:process>
  <bpmndi:BPMNDiagram id="BPMNDiagram_1">
    <bpmndi:BPMNPlane id="BPMNPlane_1" bpmnElement="Process_1">
      <bpmndi:BPMNShape id="_BPMNShape_StartEvent_1" bpmnElement="StartEvent_1">
        <dc:Bounds x="170" y="100" width="36" height="36" />
      </bpmndi:BPMNShape>
    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn:definitions>`;

const modeler = new Modeler({
  container: canvas
});

const NS_BPMN = "http://www.omg.org/spec/BPMN/20100524/MODEL";
const NS_BPMNDI = "http://www.omg.org/spec/BPMN/20100524/DI";
const NS_DC = "http://www.omg.org/spec/DD/20100524/DC";
const NS_DI = "http://www.omg.org/spec/DD/20100524/DI";

type Bounds = { x: number; y: number; width: number; height: number };

function getFirstElByLocalName(parent: ParentNode, localName: string): Element | null {
  const byNs = (parent as Document).getElementsByTagNameNS
    ? (parent as Document).getElementsByTagNameNS(NS_BPMN, localName)
    : null;
  if (byNs && byNs.length) return byNs[0];
  const plain = (parent as Document).getElementsByTagName(localName);
  if (plain && plain.length) return plain[0];
  const prefixed = (parent as Document).getElementsByTagName(`bpmn:${localName}`);
  if (prefixed && prefixed.length) return prefixed[0];
  return null;
}

function listElsIn(parent: ParentNode, localName: string): Element[] {
  // Use tagNameNS when possible (default namespace documents).
  const out: Element[] = [];
  const doc = parent as Document;
  const byNs = doc.getElementsByTagNameNS ? doc.getElementsByTagNameNS(NS_BPMN, localName) : null;
  if (byNs) {
    for (let i = 0; i < byNs.length; i++) out.push(byNs[i]);
    return out;
  }
  const plain = doc.getElementsByTagName(localName);
  for (let i = 0; i < plain.length; i++) out.push(plain[i]);
  return out;
}

function addBasicDiForLinearProcess(xml: string): string {
  const parser = new DOMParser();
  const doc = parser.parseFromString(xml, "text/xml");
  if (doc.getElementsByTagName("parsererror").length) {
    throw new Error("Invalid XML (parsererror)");
  }

  const definitions = doc.documentElement;
  if (!definitions || !/definitions$/i.test(definitions.localName)) {
    throw new Error("Expected <definitions> as document element");
  }

  // Ensure namespaces needed for DI.
  if (!definitions.getAttribute("xmlns:bpmndi")) {
    definitions.setAttribute("xmlns:bpmndi", NS_BPMNDI);
  }
  if (!definitions.getAttribute("xmlns:dc")) {
    definitions.setAttribute("xmlns:dc", NS_DC);
  }
  if (!definitions.getAttribute("xmlns:di")) {
    definitions.setAttribute("xmlns:di", NS_DI);
  }

  const process = getFirstElByLocalName(doc, "process");
  if (!process) {
    throw new Error("No <process> found");
  }
  const processId = process.getAttribute("id") || "Process_1";

  // Collect nodes
  const nodes: Element[] = [];
  // NOTE: Keep this list in sync with the BPMN we emit.
  // If an element type is not collected here, we will not create BPMNDI shapes for it,
  // and bpmn-js will not render it.
  for (const name of [
    "startEvent",
    "endEvent",
    "task",
    "serviceTask",
    "exclusiveGateway",
    "parallelGateway",
    "subProcess"
  ]) {
    const els = (process as unknown as Document).getElementsByTagNameNS
      ? process.getElementsByTagNameNS(NS_BPMN, name)
      : process.getElementsByTagName(name);
    for (let i = 0; i < els.length; i++) nodes.push(els[i]);
  }
  const nodeById = new Map<string, Element>();
  for (const n of nodes) {
    const id = n.getAttribute("id");
    if (id) nodeById.set(id, n);
  }

  // Collect sequence flows
  const flows: { id: string; source: string; target: string; name: string }[] = [];
  const flowEls = (process as unknown as Document).getElementsByTagNameNS
    ? process.getElementsByTagNameNS(NS_BPMN, "sequenceFlow")
    : process.getElementsByTagName("sequenceFlow");
  for (let i = 0; i < flowEls.length; i++) {
    const el = flowEls[i];
    const id = el.getAttribute("id") || `Flow_${i + 1}`;
    const source = el.getAttribute("sourceRef") || "";
    const target = el.getAttribute("targetRef") || "";
    const name = el.getAttribute("name") || "";
    if (source && target) {
      flows.push({ id, source, target, name });
    }
  }

  const outgoing = new Map<string, { flowId: string; target: string; name: string }[]>();
  for (const f of flows) {
    const arr = outgoing.get(f.source) ?? [];
    arr.push({ flowId: f.id, target: f.target, name: f.name });
    outgoing.set(f.source, arr);
  }

  // Prefer "yes/then/loop" ahead of "no/else/exit" for layout + lane assignment.
  function flowPriority(n: string): number {
    const s = (n || "").toLowerCase();
    if (s === "yes" || s === "then" || s === "loop") return 0;
    if (s === "no" || s === "else" || s === "exit") return 10;
    if (s === "next") return 20;
    return 5;
  }
  for (const [k, arr] of outgoing.entries()) {
    arr.sort((a, b) => {
      const pa = flowPriority(a.name);
      const pb = flowPriority(b.name);
      if (pa !== pb) return pa - pb;
      return a.target.localeCompare(b.target);
    });
    outgoing.set(k, arr);
  }

  // Determine a best-effort layout order by traversing the flow graph starting at the first start event.
  // This is still simplistic (we lay things out in a single column), but it ensures gateways and loop
  // back-edges get shapes so they are visible.
  const start = getFirstElByLocalName(process, "startEvent");
  const order: string[] = [];
  const visited = new Set<string>();
  const startId = start?.getAttribute("id");
  if (startId) {
    visited.add(startId);

    const q: string[] = [startId];
    while (q.length) {
      const cur = q.shift()!;
      order.push(cur);
      const outs = outgoing.get(cur) ?? [];
      for (const o of outs) {
        if (!nodeById.has(o.target)) continue;
        if (visited.has(o.target)) continue;
        visited.add(o.target);
        q.push(o.target);
      }
    }
  }

  for (const id of nodeById.keys()) {
    if (!visited.has(id)) order.push(id);
  }

  // Build bounds for shapes.
  // Goal: reduce overlapping arrows by spreading branches into lanes and routing loop back-edges
  // around the left side with extra bendpoints.
  const boundsById = new Map<string, Bounds>();
  const indexById = new Map<string, number>();
  for (let i = 0; i < order.length; i++) indexById.set(order[i], i);

  // Lane assignment (best-effort): when a node fans out, keep the first outgoing in the same lane
  // and push additional outgoing targets into new lanes.
  const laneById = new Map<string, number>();
  const startLane = 0;
  if (startId) laneById.set(startId, startLane);
  let nextLane = 1;

  for (const cur of order) {
    const curLane = laneById.get(cur) ?? 0;
    const outs = outgoing.get(cur) ?? [];
    for (let i = 0; i < outs.length; i++) {
      const target = outs[i].target;
      if (!nodeById.has(target)) continue;
      if (laneById.has(target)) continue;
      laneById.set(target, i === 0 ? curLane : nextLane++);
    }
  }
  for (const id of nodeById.keys()) {
    if (!laneById.has(id)) laneById.set(id, 0);
  }

  // Layout constants
  const baseX = 80;
  const baseY = 70;
  const laneWidth = 260;
  const rowHeight = 120;

  for (const id of order) {
    const el = nodeById.get(id);
    if (!el) continue;
    const local = el.localName;
    const isEvent = local === "startEvent" || local === "endEvent";
    const isGateway = local === "exclusiveGateway" || local === "parallelGateway";
    const width = isEvent ? 36 : isGateway ? 56 : 170;
    const height = isEvent ? 36 : isGateway ? 56 : 90;

    const lane = laneById.get(id) ?? 0;
    const row = indexById.get(id) ?? 0;
    const x = baseX + lane * laneWidth;
    const y = baseY + row * rowHeight;
    boundsById.set(id, { x, y, width, height });
  }

  // Create BPMNDI
  const diagram = doc.createElementNS(NS_BPMNDI, "bpmndi:BPMNDiagram");
  diagram.setAttribute("id", `BPMNDiagram_${processId}`);

  const plane = doc.createElementNS(NS_BPMNDI, "bpmndi:BPMNPlane");
  plane.setAttribute("id", `BPMNPlane_${processId}`);
  plane.setAttribute("bpmnElement", processId);
  diagram.appendChild(plane);

  // Shapes
  for (const [id, b] of boundsById.entries()) {
    const shape = doc.createElementNS(NS_BPMNDI, "bpmndi:BPMNShape");
    shape.setAttribute("id", `BPMNShape_${id}`);
    shape.setAttribute("bpmnElement", id);

    const bounds = doc.createElementNS(NS_DC, "dc:Bounds");
    bounds.setAttribute("x", String(b.x));
    bounds.setAttribute("y", String(b.y));
    bounds.setAttribute("width", String(b.width));
    bounds.setAttribute("height", String(b.height));
    shape.appendChild(bounds);
    plane.appendChild(shape);
  }

  // Edges
  let backEdgeIdx = 0;
  for (const f of flows) {
    const src = boundsById.get(f.source);
    const dst = boundsById.get(f.target);
    if (!src || !dst) continue;

    const edge = doc.createElementNS(NS_BPMNDI, "bpmndi:BPMNEdge");
    edge.setAttribute("id", `BPMNEdge_${f.id}`);
    edge.setAttribute("bpmnElement", f.id);

    const srcEl = nodeById.get(f.source);
    const dstEl = nodeById.get(f.target);
    const srcLocal = srcEl?.localName;
    const dstLocal = dstEl?.localName;
    const srcIsGateway = srcLocal === "exclusiveGateway" || srcLocal === "parallelGateway";
    const dstIsGateway = dstLocal === "exclusiveGateway" || dstLocal === "parallelGateway";

    const flowName = (f.name || "").toLowerCase();
    const isYesFlow = flowName === "yes" || flowName === "then" || flowName === "loop";

        // Sort flows to get a readable mainline.
        // For if/elseif: keep "no/else" going down; branch "yes/then" to the right.
        // For loops: keep "loop" going down; "exit" peels off.
        function flowPriority(n: string): number {
          const s = (n || "").toLowerCase();
          if (s === "loop") return 0;
          if (s === "no" || s === "else") return 1;
          if (s === "exit") return 2;
          if (s === "yes" || s === "then") return 10;
          if (s === "next") return 20;
          return 5;
        }
    const srcRow = indexById.get(f.source) ?? 0;
    const dstRow = indexById.get(f.target) ?? 0;
    const isBackEdge = dstRow <= srcRow;
    const srcLane = laneById.get(f.source) ?? 0;
    const dstLane = laneById.get(f.target) ?? 0;
    const isCrossLane = srcLane !== dstLane;

    const waypoints: Array<{ x: number; y: number }> = [];
    waypoints.push({ x: x1, y: y1 });

    if (isBackEdge) {
      // Route loop-back edges around the left side so they don't overlap vertical flows.
      const loopOffset = 140 + backEdgeIdx * 40;
      backEdgeIdx++;
      const leftX = Math.min(src.x, dst.x) - loopOffset;
      waypoints.push({ x: leftX, y: y1 });
      waypoints.push({ x: leftX, y: y2 });
      waypoints.push({ x: x2, y: y2 });
    } else if (isCrossLane) {
      // Cross-lane forward flow: go down a bit, go sideways, then down into the target.
      const midY = y1 + 24;
      waypoints.push({ x: x1, y: midY });
      waypoints.push({ x: x2, y: midY });
      waypoints.push({ x: x2, y: y2 });
    } else {
      // Same-lane forward flow: straight down.
      waypoints.push({ x: x2, y: y2 });
    }

    for (const p of waypoints) {
      const wp = doc.createElementNS(NS_DI, "di:waypoint");
      wp.setAttribute("x", String(p.x));
      wp.setAttribute("y", String(p.y));
      edge.appendChild(wp);
    }
    plane.appendChild(edge);
  }

  definitions.appendChild(diagram);
  return new XMLSerializer().serializeToString(doc);
}

function hasBpmnDi(xml: string): boolean {
  return /<\w*:?BPMNDiagram\b|<\w*:?BPMNPlane\b|xmlns:bpmndi=/.test(xml);
}

async function ensureDiagramRenders(xml: string): Promise<string> {
        function isGatewayId(id: string): boolean {
          const el = nodeById.get(id);
          const local = el?.localName;
          return local === "exclusiveGateway" || local === "parallelGateway";
        }

        function isDownPreferredFlowName(name: string): boolean {
          const s = (name || "").toLowerCase();
          return s === "no" || s === "else" || s === "loop" || s === "exit";
        }

  if (hasBpmnDi(xml)) return xml;
  // bpmn-js imports semantic BPMN fine, but without DI it renders blank.
  // Add a simple DI (shapes + edges) so arrows render.
  return addBasicDiForLinearProcess(xml);
}

let pendingSyncTimer: number | null = null;
async function syncXmlFromDiagram(opts?: { force?: boolean }) {
  const force = opts?.force ?? false;
  try {
    const { xml } = await modeler.saveXML({ format: true });
    if (!xml) return;
    currentXml = xml;
    if (!editorDirty || force) {
      editorXml = xml;
      editorDirty = false;
    }
    if (activeTab === "bpmn") {
      renderPanel();
    }
  } catch (e) {
    console.error(e);
  }
}

// Keep XML panel in sync as the diagram changes.
modeler.on("commandStack.changed", () => {
  if (pendingSyncTimer != null) {
    window.clearTimeout(pendingSyncTimer);
  }
  pendingSyncTimer = window.setTimeout(() => {
    pendingSyncTimer = null;
    syncXmlFromDiagram();
  }, 250);
});

async function loadXml(xml: string) {
  try {
    const xmlToImport = await ensureDiagramRenders(xml);
    await modeler.importXML(xmlToImport);
    const canvasSvc = modeler.get("canvas") as { zoom: (arg: string) => void };
    canvasSvc.zoom("fit-viewport");
    currentXml = xmlToImport;
    editorXml = xmlToImport;
    editorDirty = false;
    lastRust = "";
    lastTs = "";
    lastValidate = null;
    renderPanel();
    setStatus("Loaded BPMN", "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Failed to load BPMN: ${toErrorString(e)}`, "err");
  }
}

async function exportXml(): Promise<string> {
  const { xml } = await modeler.saveXML({ format: true });
  if (!xml) {
    throw new Error("Failed to export BPMN XML");
  }
  return xml;
}

async function refreshDiskSignature(path: string): Promise<string | null> {
  try {
    const disk = await invoke<string>("read_text_file", { path });
    const h = hashStringFNV1a(disk);
    lastKnownDiskHash = h;
    return h;
  } catch {
    // File may not exist yet.
    lastKnownDiskHash = null;
    return null;
  }
}

async function saveToPath(path: string, xml: string, opts?: { checkExternal?: boolean }) {
  const checkExternal = opts?.checkExternal ?? true;
  if (checkExternal) {
    let diskHash: string | null = null;
    try {
      const disk = await invoke<string>("read_text_file", { path });
      diskHash = hashStringFNV1a(disk);
    } catch {
      diskHash = null;
    }

    if (diskHash && lastKnownDiskHash && diskHash !== lastKnownDiskHash) {
      const choice = await showChoiceModal({
        title: "File changed on disk",
        body: "This file was modified on disk since you opened/saved it. Overwrite it?",
        choices: [
          { id: "overwrite", label: "Overwrite", variant: "danger" },
          { id: "cancel", label: "Cancel", variant: "default" }
        ],
        dismissId: "cancel"
      });
      if (choice !== "overwrite") {
        throw new Error("Save canceled");
      }
    }
  }

  await invoke<void>("write_text_file", { path, contents: xml });
  setCurrentFilePath(path);
  lastSavedXml = xml;
  lastKnownDiskHash = hashStringFNV1a(xml);
  editorDirty = false;
  editorXml = xml;
  renderPanel();
}

async function saveAsFlow(xml: string): Promise<string | null> {
  const defaultName = currentFilePath ? currentFilePath : (lastDir ? `${lastDir}\\diagram.bpmn` : "diagram.bpmn");
  const target = await save({
    filters: [{ name: "BPMN", extensions: ["bpmn", "xml"] }],
    defaultPath: defaultName
  });
  if (!target) return null;

  let exists = false;
  try {
    await invoke<string>("read_text_file", { path: target });
    exists = true;
  } catch {
    exists = false;
  }

  if (exists) {
    const choice = await showChoiceModal({
      title: "Overwrite file?",
      body: "That file already exists. Overwrite it?",
      choices: [
        { id: "overwrite", label: "Overwrite", variant: "danger" },
        { id: "cancel", label: "Cancel", variant: "default" }
      ],
      dismissId: "cancel"
    });
    if (choice !== "overwrite") return null;
  }

  await saveToPath(target, xml, { checkExternal: false });
  return target;
}

async function ensureSavedForAction(): Promise<{ path: string | null; xml: string } | null> {
  const xml = await exportXml();
  currentXml = xml;

  if (!currentFilePath) {
    const savedPath = await saveAsFlow(xml);
    if (!savedPath) return null;
    return { path: savedPath, xml };
  }

  // Refresh disk signature on demand if we haven't yet.
  if (!lastKnownDiskHash) {
    await refreshDiskSignature(currentFilePath);
  }

  // Always save before action (requested behavior), but follow Save rules.
  await saveToPath(currentFilePath, xml, { checkExternal: true });
  return { path: currentFilePath, xml };
}

async function isDefaultTemplate(): Promise<boolean> {
  if (!defaultTemplateCanonical) return false;
  const xml = await exportXml();
  return normalizeXmlForCompare(xml) === normalizeXmlForCompare(defaultTemplateCanonical);
}

async function maybePromptToSaveBeforeDestructive(opts: {
  actionName: string;
  saveLabel: string;
}): Promise<"save" | "discard" | "cancel"> {
  const defaultOk = await isDefaultTemplate();
  if (defaultOk) return "discard"; // safe: nothing important

  const choice = await showChoiceModal({
    title: `${opts.actionName}: Unsaved or non-default work`,
    body: "Your current diagram is not the default template. What do you want to do?",
    choices: [
      { id: "save", label: opts.saveLabel, variant: "primary" },
      { id: "discard", label: "Discard", variant: "danger" },
      { id: "cancel", label: "Cancel", variant: "default" }
    ],
    dismissId: "cancel"
  });
  if (choice === "save") return "save";
  if (choice === "discard") return "discard";
  return "cancel";
}

newBtn.addEventListener("click", () => {
  (async () => {
    try {
      const decision = await maybePromptToSaveBeforeDestructive({
        actionName: "New",
        saveLabel: "Save & New"
      });
      if (decision === "cancel") return;
      if (decision === "save") {
        const saved = await ensureSavedForAction();
        if (!saved) return;
      }
      setCurrentFilePath(null);
      lastKnownDiskHash = null;
      lastSavedXml = "";
      await loadXml(DEFAULT_XML);
      setActiveTab("bpmn");
      setStatus("New diagram", "neutral");
    } catch (e) {
      console.error(e);
      setStatus(`New failed: ${toErrorString(e)}`, "err");
    }
  })();
});

openNativeBtn.addEventListener("click", async () => {
  try {
    setStatus("Opening…", "neutral");
    const decision = await maybePromptToSaveBeforeDestructive({
      actionName: "Open",
      saveLabel: "Save & Open"
    });
    if (decision === "cancel") return;
    if (decision === "save") {
      const saved = await ensureSavedForAction();
      if (!saved) return;
    }

    const selected = await open({
      multiple: false,
      filters: [{ name: "BPMN", extensions: ["bpmn", "xml"] }],
      defaultPath: lastDir ?? undefined
    });
    if (!selected || Array.isArray(selected)) return;

    const xml = await invoke<string>("read_text_file", { path: selected });
    setCurrentFilePath(selected);
    lastSavedXml = xml;
    lastKnownDiskHash = hashStringFNV1a(xml);
    await loadXml(xml);
    setActiveTab("bpmn");
    setStatus(`Loaded ${selected}`, "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Open failed: ${toErrorString(e)}`, "err");
  }
});

saveNativeBtn.addEventListener("click", async () => {
  try {
    setStatus("Saving…", "neutral");
    const xml = await exportXml();
    currentXml = xml;
    editorXml = xml;
    editorDirty = false;
    renderPanel();

    if (!currentFilePath) {
      // Save is disabled until a file path exists, but keep this safe.
      setStatus("Save is disabled until a file is opened (use Save As)", "warn");
      return;
    }

    if (!lastKnownDiskHash) {
      await refreshDiskSignature(currentFilePath);
    }
    await saveToPath(currentFilePath, xml, { checkExternal: true });
    setStatus(`Saved ${currentFilePath}`, "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Save failed: ${toErrorString(e)}`, "err");
  }
});

saveAsNativeBtn.addEventListener("click", async () => {
  try {
    setStatus("Saving As…", "neutral");
    const xml = await exportXml();
    currentXml = xml;
    const target = await saveAsFlow(xml);
    if (!target) return;
    setStatus(`Saved ${target}`, "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Save As failed: ${toErrorString(e)}`, "err");
  }
});

convertRustBtn.addEventListener("click", async () => {
  try {
    setStatus("Converting to Rust…", "neutral");
    const saved = await ensureSavedForAction();
    if (!saved) return;
    lastRust = await invoke<string>("bpmn_to_rust", { xml: saved.xml });
    rustModel.setValue(lastRust);
    setActiveTab("rust");
    setStatus("Converted to Rust", "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Convert to Rust failed: ${toErrorString(e)}`, "err");
  }
});

convertTsBtn.addEventListener("click", async () => {
  try {
    setStatus("Converting to TS…", "neutral");
    const saved = await ensureSavedForAction();
    if (!saved) return;
    // Requested behavior: BPMN -> Rust -> TS (convert Rust output, not BPMN directly).
    lastRust = await invoke<string>("bpmn_to_rust", { xml: saved.xml });
    rustModel.setValue(lastRust);
    lastTs = await invoke<string>("rust_to_ts", { rust: lastRust });
    tsModel.setValue(lastTs);
    setActiveTab("ts");
    setStatus("Converted to TS", "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Convert to TS failed: ${toErrorString(e)}`, "err");
  }
});

syncFromRustBtn.addEventListener("click", async () => {
  try {
    const rustText = rustModel.getValue();
    if (!rustText.trim()) {
      setStatus("Sync ← Rust: convert to Rust first", "warn");
      return;
    }
    const decision = await maybePromptToSaveBeforeDestructive({
      actionName: "Sync ← Rust",
      saveLabel: "Save & Sync"
    });
    if (decision === "cancel") return;
    if (decision === "save") {
      const saved = await ensureSavedForAction();
      if (!saved) return;
    }

    setStatus("Syncing from Rust…", "neutral");
    const nextXml = await invoke<string>("rust_to_bpmn", { rust: rustText });
    await loadXml(nextXml);
    setActiveTab("bpmn");
    setStatus("Synced diagram from Rust", "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Sync ← Rust failed: ${toErrorString(e)}`, "err");
  }
});

syncFromTsBtn.addEventListener("click", async () => {
  try {
    const tsText = tsModel.getValue();
    if (!tsText.trim()) {
      setStatus("Sync ← TS: convert to TS first", "warn");
      return;
    }
    const decision = await maybePromptToSaveBeforeDestructive({
      actionName: "Sync ← TS",
      saveLabel: "Save & Sync"
    });
    if (decision === "cancel") return;
    if (decision === "save") {
      const saved = await ensureSavedForAction();
      if (!saved) return;
    }

    setStatus("Syncing from TS…", "neutral");
    const nextXml = await invoke<string>("ts_to_bpmn", { ts: tsText });
    await loadXml(nextXml);
    setActiveTab("bpmn");
    setStatus("Synced diagram from TS", "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Sync ← TS failed: ${toErrorString(e)}`, "err");
  }
});

validateBtn.addEventListener("click", async () => {
  try {
    setStatus("Validating…", "neutral");
    const saved = await ensureSavedForAction();
    if (!saved) return;
    lastValidate = await invoke<ValidateResult>("validate_roundtrip", { xml: saved.xml });
    lastRust = lastValidate.rust_direct;
    setActiveTab("out");
    // Requested UX: blue when not valid, green when valid.
    setStatus(lastValidate.ok ? "Validate OK" : "Validate FAILED", lastValidate.ok ? "ok" : "warn");

    const pre = document.createElement("pre");
    pre.textContent = [
      `OK: ${lastValidate.ok}`,
      "",
      "--- stdout (direct) ---",
      lastValidate.stdout_direct,
      "--- stdout (roundtrip) ---",
      lastValidate.stdout_roundtrip,
      "--- rust (direct) ---",
      lastValidate.rust_direct,
      "--- rust (roundtrip) ---",
      lastValidate.rust_roundtrip,
      "--- bpmn (roundtrip) ---",
      lastValidate.bpmn_roundtrip
    ].join("\n");

    await showChoiceModal({
      title: lastValidate.ok ? "Validate OK" : "Validate FAILED",
      body: pre,
      choices: [{ id: "close", label: "Close", variant: "primary" }],
      dismissId: "close"
    });
  } catch (e) {
    console.error(e);
    setStatus(`Validate failed: ${toErrorString(e)}`, "err");
  }
});

// BPMN XML editor UX
bpmnEditor.addEventListener("input", () => {
  editorXml = bpmnEditor.value;
  editorDirty = true;
});

applyXmlBtn.addEventListener("click", async () => {
  try {
    const xml = bpmnEditor.value;
    await loadXml(xml);
    setActiveTab("bpmn");
    setStatus("Applied BPMN XML", "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Apply failed: ${toErrorString(e)}`, "err");
  }
});

syncFromDiagramBtn.addEventListener("click", async () => {
  await syncXmlFromDiagram({ force: true });
  setActiveTab("bpmn");
  setStatus("Synced XML from diagram", "neutral");
});

tabBpmn.addEventListener("click", () => setActiveTab("bpmn"));
tabRust.addEventListener("click", () => setActiveTab("rust"));
tabTs.addEventListener("click", () => setActiveTab("ts"));
tabOut.addEventListener("click", () => setActiveTab("out"));

// Initial setup
loadLastDirFromStorage();
updateButtonStates();

window.addEventListener("error", (ev) => {
  const msg = (ev.error instanceof Error ? ev.error.stack ?? ev.error.message : String(ev.message ?? ev.error)) || "Unknown error";
  console.error(ev.error);
  setStatus(`Error: ${msg}`, "err");
});

window.addEventListener("unhandledrejection", (ev) => {
  const reason = (ev.reason instanceof Error ? ev.reason.stack ?? ev.reason.message : toErrorString(ev.reason)) || "Unknown rejection";
  console.error(ev.reason);
  setStatus(`Unhandled: ${reason}`, "err");
});

(async () => {
  try {
    await loadXml(DEFAULT_XML);
    // Record the canonical default template as bpmn-js exports it.
    try {
      defaultTemplateCanonical = await exportXml();
      lastSavedXml = defaultTemplateCanonical;
    } catch {
      defaultTemplateCanonical = DEFAULT_XML;
      lastSavedXml = DEFAULT_XML;
    }
    setActiveTab("bpmn");
  } catch (e) {
    console.error(e);
    showStartupError("Startup failed", toErrorString(e));
  }
})();
