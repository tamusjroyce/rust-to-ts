import Modeler from "bpmn-js/lib/Modeler";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

const canvas = document.getElementById("canvas") as HTMLDivElement;
const status = document.getElementById("status") as HTMLDivElement;

const newBtn = document.getElementById("newBtn") as HTMLButtonElement;
const openNativeBtn = document.getElementById("openNativeBtn") as HTMLButtonElement;
const saveNativeBtn = document.getElementById("saveNativeBtn") as HTMLButtonElement;
const convertRustBtn = document.getElementById("convertRustBtn") as HTMLButtonElement;
const convertTsBtn = document.getElementById("convertTsBtn") as HTMLButtonElement;
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
const panel = document.getElementById("panel") as HTMLPreElement;

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
let lastRust: string = "";
let lastTs: string = "";
let lastValidate: ValidateResult | null = null;

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
    panel.style.display = "none";
    bpmnEditor.value = editorXml || currentXml || "";
    return;
  }
  bpmnEditor.style.display = "none";
  panel.style.display = "block";
  if (activeTab === "rust") {
    panel.textContent = lastRust || "(convert to Rust first)";
    return;
  }
  if (activeTab === "ts") {
    panel.textContent = lastTs || "(convert to TS first)";
    return;
  }
  // out
  if (!lastValidate) {
    panel.textContent = "(validate to see results)";
    return;
  }
  const v = lastValidate;
  panel.textContent = [
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
  for (const name of ["startEvent", "serviceTask", "endEvent"]) {
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
  const flows: { id: string; source: string; target: string }[] = [];
  const flowEls = (process as unknown as Document).getElementsByTagNameNS
    ? process.getElementsByTagNameNS(NS_BPMN, "sequenceFlow")
    : process.getElementsByTagName("sequenceFlow");
  for (let i = 0; i < flowEls.length; i++) {
    const el = flowEls[i];
    const id = el.getAttribute("id") || `Flow_${i + 1}`;
    const source = el.getAttribute("sourceRef") || "";
    const target = el.getAttribute("targetRef") || "";
    if (source && target) {
      flows.push({ id, source, target });
    }
  }

  const outgoing = new Map<string, { flowId: string; target: string }[]>();
  for (const f of flows) {
    const arr = outgoing.get(f.source) ?? [];
    arr.push({ flowId: f.id, target: f.target });
    outgoing.set(f.source, arr);
  }

  // Determine a best-effort linear order by following sequence flows from the first start event.
  const start = getFirstElByLocalName(process, "startEvent");
  const order: string[] = [];
  const visited = new Set<string>();
  const startId = start?.getAttribute("id");
  if (startId) {
    order.push(startId);
    visited.add(startId);
    let cur = startId;
    while (true) {
      const outs = outgoing.get(cur) ?? [];
      const next = outs.find(o => !visited.has(o.target) && nodeById.has(o.target));
      if (!next) break;
      order.push(next.target);
      visited.add(next.target);
      cur = next.target;
    }
  }

  for (const id of nodeById.keys()) {
    if (!visited.has(id)) order.push(id);
  }

  // Build bounds for shapes (vertical layout)
  const boundsById = new Map<string, Bounds>();
  const x = 60;
  let y = 60;
  const gap = 120;

  for (const id of order) {
    const el = nodeById.get(id);
    if (!el) continue;
    const local = el.localName;
    const isEvent = local === "startEvent" || local === "endEvent";
    const width = isEvent ? 36 : 140;
    const height = isEvent ? 36 : 80;
    boundsById.set(id, { x, y, width, height });
    y += gap;
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
  for (const f of flows) {
    const src = boundsById.get(f.source);
    const dst = boundsById.get(f.target);
    if (!src || !dst) continue;

    const edge = doc.createElementNS(NS_BPMNDI, "bpmndi:BPMNEdge");
    edge.setAttribute("id", `BPMNEdge_${f.id}`);
    edge.setAttribute("bpmnElement", f.id);

    const x1 = src.x + src.width / 2;
    const y1 = src.y + src.height;
    const x2 = dst.x + dst.width / 2;
    const y2 = dst.y;

    const wp1 = doc.createElementNS(NS_DI, "di:waypoint");
    wp1.setAttribute("x", String(x1));
    wp1.setAttribute("y", String(y1));
    const wp2 = doc.createElementNS(NS_DI, "di:waypoint");
    wp2.setAttribute("x", String(x2));
    wp2.setAttribute("y", String(y2));
    edge.appendChild(wp1);
    edge.appendChild(wp2);
    plane.appendChild(edge);
  }

  definitions.appendChild(diagram);
  return new XMLSerializer().serializeToString(doc);
}

function hasBpmnDi(xml: string): boolean {
  return /<\w*:?BPMNDiagram\b|<\w*:?BPMNPlane\b|xmlns:bpmndi=/.test(xml);
}

async function ensureDiagramRenders(xml: string): Promise<string> {
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

newBtn.addEventListener("click", () => {
  currentFilePath = null;
  loadXml(DEFAULT_XML);
});

openNativeBtn.addEventListener("click", async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "BPMN", extensions: ["bpmn", "xml"] }]
    });
    if (!selected || Array.isArray(selected)) return;

    const xml = await invoke<string>("read_text_file", { path: selected });
    currentFilePath = selected;
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
    const xml = await exportXml();
    currentXml = xml;
    editorXml = xml;
    editorDirty = false;
    renderPanel();

    // If a file is already open, save back to it (works even if dialogs are flaky).
    if (currentFilePath) {
      await invoke<void>("write_text_file", { path: currentFilePath, contents: xml });
      setStatus(`Saved ${currentFilePath}`, "neutral");
      return;
    }

    const target = await save({
      filters: [{ name: "BPMN", extensions: ["bpmn"] }],
      defaultPath: "diagram.bpmn"
    });
    if (!target) return;

    await invoke<void>("write_text_file", { path: target, contents: xml });
    currentFilePath = target;
    setStatus(`Saved ${target}`, "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Save failed: ${toErrorString(e)}`, "err");
  }
});

convertRustBtn.addEventListener("click", async () => {
  try {
    const xml = await exportXml();
    currentXml = xml;
    lastRust = await invoke<string>("bpmn_to_rust", { xml });
    setActiveTab("rust");
    setStatus("Converted to Rust", "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Convert to Rust failed: ${toErrorString(e)}`, "err");
  }
});

convertTsBtn.addEventListener("click", async () => {
  try {
    const xml = await exportXml();
    currentXml = xml;
    lastTs = await invoke<string>("bpmn_to_ts", { xml });
    setActiveTab("ts");
    setStatus("Converted to TS", "neutral");
  } catch (e) {
    console.error(e);
    setStatus(`Convert to TS failed: ${toErrorString(e)}`, "err");
  }
});

validateBtn.addEventListener("click", async () => {
  try {
    const xml = await exportXml();
    currentXml = xml;
    lastValidate = await invoke<ValidateResult>("validate_roundtrip", { xml });
    lastRust = lastValidate.rust_direct;
    setActiveTab("out");
    // Requested UX: blue when not valid, green when valid.
    setStatus(lastValidate.ok ? "Validate OK" : "Validate FAILED", lastValidate.ok ? "ok" : "warn");
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

loadXml(DEFAULT_XML);
setActiveTab("bpmn");
