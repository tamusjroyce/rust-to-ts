import Modeler from "https://esm.sh/bpmn-js@18.5.0/lib/Modeler?bundle";

const canvas = document.getElementById("canvas") as HTMLDivElement;
const status = document.getElementById("status") as HTMLDivElement;

const newBtn = document.getElementById("newBtn") as HTMLButtonElement;
const saveBtn = document.getElementById("saveBtn") as HTMLButtonElement;
const openFile = document.getElementById("openFile") as HTMLInputElement;

function setStatus(msg: string) {
  status.textContent = msg;
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
  container: canvas,
});

async function loadXml(xml: string) {
  try {
    await modeler.importXML(xml);
    const canvasSvc = modeler.get("canvas") as { zoom: (arg: string) => void };
    canvasSvc.zoom("fit-viewport");
    setStatus("Loaded BPMN");
  } catch (e) {
    console.error(e);
    setStatus("Failed to load BPMN (see console)");
  }
}

async function exportXml(): Promise<string> {
  const { xml } = await modeler.saveXML({ format: true });
  return xml;
}

function download(filename: string, contents: string, mime = "application/xml") {
  const blob = new Blob([contents], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

newBtn.addEventListener("click", () => {
  loadXml(DEFAULT_XML);
});

saveBtn.addEventListener("click", async () => {
  try {
    const xml = await exportXml();
    download("diagram.bpmn", xml);
    setStatus("Saved diagram.bpmn");
  } catch (e) {
    console.error(e);
    setStatus("Save failed (see console)");
  }
});

openFile.addEventListener("change", async () => {
  const file = openFile.files?.[0];
  if (!file) return;
  try {
    const xml = await file.text();
    await loadXml(xml);
    setStatus(`Loaded ${file.name}`);
  } catch (e) {
    console.error(e);
    setStatus("Open failed (see console)");
  } finally {
    openFile.value = "";
  }
});

loadXml(DEFAULT_XML);
