use std::collections::{HashMap, HashSet};

use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BpmnProcess {
    pub id: String,
    pub name: Option<String>,
    pub nodes: Vec<BpmnNode>,
    pub flows: Vec<BpmnSequenceFlow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustFieldSig {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustParamSig {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BpmnNode {
    StartEvent { id: String, name: Option<String> },
    EndEvent { id: String, name: Option<String> },
    Task { id: String, name: Option<String> },
    ServiceTask { id: String, name: Option<String> },
    ExclusiveGateway { id: String, name: Option<String> },
    ParallelGateway { id: String, name: Option<String> },
    /// Non-standard node used to losslessly round-trip the AST-v2 Rust model.
    /// Emits as `<rustType .../>` in the XML.
    RustType {
        id: String,
        name: String,
        kind: String,
        fields: Vec<RustFieldSig>,
    },
    /// Non-standard node used to losslessly round-trip the AST-v2 Rust model.
    /// Emits as `<rustFunction .../>` in the XML.
    RustFunction {
        id: String,
        name: String,
        params: Vec<RustParamSig>,
        return_type: Option<String>,
        body: Vec<String>,
    },
    Unknown { tag: String, id: Option<String>, name: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BpmnSequenceFlow {
    pub id: String,
    pub source_ref: String,
    pub target_ref: String,
    pub name: Option<String>,
}

fn local_name(qname: &[u8]) -> &str {
    let s = std::str::from_utf8(qname).unwrap_or("");
    s.rsplit(':').next().unwrap_or(s)
}

fn attr_string(e: &quick_xml::events::BytesStart<'_>, key: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .and_then(|a| std::str::from_utf8(&a.value).ok().map(|s| s.to_string()))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn kv_list_encode(items: &[(String, String)]) -> String {
    // Deterministic, lossless for simple identifier-like strings.
    // Format: name:ty;name2:ty2
    let mut out = String::new();
    for (idx, (k, v)) in items.iter().enumerate() {
        if idx > 0 {
            out.push(';');
        }
        out.push_str(k);
        out.push(':');
        out.push_str(v);
    }
    out
}

fn kv_list_decode(s: &str) -> Vec<(String, String)> {
    let s = s.trim();
    if s.is_empty() {
        return Vec::new();
    }
    s.split(';')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            let (k, v) = part.split_once(':')?;
            Some((k.trim().to_string(), v.trim().to_string()))
        })
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err("hex string must have even length".to_string());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let hex_val = |c: u8| -> Result<u8, String> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            _ => Err(format!("invalid hex char: {}", c as char)),
        }
    };
    for i in (0..bytes.len()).step_by(2) {
        let hi = hex_val(bytes[i])?;
        let lo = hex_val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

pub fn parse_bpmn_xml(xml: &str) -> Result<BpmnProcess, String> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut process: Option<BpmnProcess> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let tag = local_name(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "process" => {
                        let id = attr_string(&e, b"id").unwrap_or_else(|| "process".to_string());
                        let name = attr_string(&e, b"name");
                        process = Some(BpmnProcess {
                            id,
                            name,
                            nodes: Vec::new(),
                            flows: Vec::new(),
                        });
                    }
                    "startEvent" => {
                        if let Some(p) = process.as_mut() {
                            p.nodes.push(BpmnNode::StartEvent {
                                id: attr_string(&e, b"id").unwrap_or_else(|| "StartEvent_1".to_string()),
                                name: attr_string(&e, b"name"),
                            });
                        }
                    }
                    "endEvent" => {
                        if let Some(p) = process.as_mut() {
                            p.nodes.push(BpmnNode::EndEvent {
                                id: attr_string(&e, b"id").unwrap_or_else(|| "EndEvent_1".to_string()),
                                name: attr_string(&e, b"name"),
                            });
                        }
                    }
                    "task" => {
                        if let Some(p) = process.as_mut() {
                            p.nodes.push(BpmnNode::Task {
                                id: attr_string(&e, b"id").unwrap_or_else(|| "Task_1".to_string()),
                                name: attr_string(&e, b"name"),
                            });
                        }
                    }
                    "serviceTask" => {
                        if let Some(p) = process.as_mut() {
                            p.nodes.push(BpmnNode::ServiceTask {
                                id: attr_string(&e, b"id").unwrap_or_else(|| "ServiceTask_1".to_string()),
                                name: attr_string(&e, b"name"),
                            });
                        }
                    }
                    "exclusiveGateway" => {
                        if let Some(p) = process.as_mut() {
                            p.nodes.push(BpmnNode::ExclusiveGateway {
                                id: attr_string(&e, b"id").unwrap_or_else(|| "ExclusiveGateway_1".to_string()),
                                name: attr_string(&e, b"name"),
                            });
                        }
                    }
                    "parallelGateway" => {
                        if let Some(p) = process.as_mut() {
                            p.nodes.push(BpmnNode::ParallelGateway {
                                id: attr_string(&e, b"id").unwrap_or_else(|| "ParallelGateway_1".to_string()),
                                name: attr_string(&e, b"name"),
                            });
                        }
                    }
                    "sequenceFlow" => {
                        if let Some(p) = process.as_mut() {
                            let id = attr_string(&e, b"id").unwrap_or_else(|| "Flow_1".to_string());
                            let source_ref = attr_string(&e, b"sourceRef")
                                .ok_or_else(|| format!("sequenceFlow {id} missing sourceRef"))?;
                            let target_ref = attr_string(&e, b"targetRef")
                                .ok_or_else(|| format!("sequenceFlow {id} missing targetRef"))?;
                            p.flows.push(BpmnSequenceFlow {
                                id,
                                source_ref,
                                target_ref,
                                name: attr_string(&e, b"name"),
                            });
                        }
                    }
                    "rustType" => {
                        if let Some(p) = process.as_mut() {
                            let id = attr_string(&e, b"id").unwrap_or_else(|| "RustType_1".to_string());
                            let name = attr_string(&e, b"name").unwrap_or_else(|| "Type".to_string());
                            let kind = attr_string(&e, b"kind").unwrap_or_else(|| "struct".to_string());
                            let fields_raw = attr_string(&e, b"fields").unwrap_or_default();
                            let fields = kv_list_decode(&fields_raw)
                                .into_iter()
                                .map(|(name, ty)| RustFieldSig { name, ty })
                                .collect();
                            p.nodes.push(BpmnNode::RustType {
                                id,
                                name,
                                kind,
                                fields,
                            });
                        }
                    }
                    "rustFunction" => {
                        if let Some(p) = process.as_mut() {
                            let id = attr_string(&e, b"id").unwrap_or_else(|| "RustFunction_1".to_string());
                            let name = attr_string(&e, b"name").unwrap_or_else(|| "function".to_string());
                            let params_raw = attr_string(&e, b"params").unwrap_or_default();
                            let params = kv_list_decode(&params_raw)
                                .into_iter()
                                .map(|(name, ty)| RustParamSig { name, ty })
                                .collect();
                            let return_type = attr_string(&e, b"returnType");

                            let body = match attr_string(&e, b"bodyHex") {
                                None => Vec::new(),
                                Some(h) if h.trim().is_empty() => Vec::new(),
                                Some(h) => {
                                    let bytes = hex_decode(&h)?;
                                    let s = String::from_utf8(bytes)
                                        .map_err(|e| format!("rustFunction bodyHex is not valid UTF-8: {e}"))?;
                                    if s.is_empty() {
                                        Vec::new()
                                    } else {
                                        s.split('\n').map(|l| l.to_string()).collect()
                                    }
                                }
                            };

                            p.nodes.push(BpmnNode::RustFunction {
                                id,
                                name,
                                params,
                                return_type,
                                body,
                            });
                        }
                    }
                    _ => {
                        // Keep unknown nodes (with identity) for best-effort retention.
                        if let Some(p) = process.as_mut() {
                            let id = attr_string(&e, b"id");
                            let name = attr_string(&e, b"name");
                            if id.is_some() || name.is_some() {
                                p.nodes.push(BpmnNode::Unknown { tag, id, name });
                            }
                        }
                    }
                }
            }
            Ok(_) => {}
            Err(e) => return Err(format!("BPMN XML parse error: {e}")),
        }
        buf.clear();
    }

    process.ok_or_else(|| "No <process> found in BPMN XML".to_string())
}

pub fn emit_bpmn_xml(proc: &BpmnProcess) -> String {
    // Minimal BPMN 2.0 envelope. Deterministic ordering.
    let mut out = String::new();
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push('\n');
    out.push_str(
        r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL" xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="Definitions_1">"#,
    );
    out.push('\n');
    out.push_str(&format!(r#"  <process id="{}""#, xml_escape(&proc.id)));
    if let Some(name) = &proc.name {
        out.push_str(&format!(r#" name="{}""#, xml_escape(name)));
    }
    out.push_str(">\n");

    for n in &proc.nodes {
        match n {
            BpmnNode::StartEvent { id, name } => {
                out.push_str(&format!(r#"    <startEvent id="{}""#, xml_escape(id)));
                if let Some(name) = name {
                    out.push_str(&format!(r#" name="{}""#, xml_escape(name)));
                }
                out.push_str("/>\n");
            }
            BpmnNode::EndEvent { id, name } => {
                out.push_str(&format!(r#"    <endEvent id="{}""#, xml_escape(id)));
                if let Some(name) = name {
                    out.push_str(&format!(r#" name="{}""#, xml_escape(name)));
                }
                out.push_str("/>\n");
            }
            BpmnNode::Task { id, name } => {
                out.push_str(&format!(r#"    <task id="{}""#, xml_escape(id)));
                if let Some(name) = name {
                    out.push_str(&format!(r#" name="{}""#, xml_escape(name)));
                }
                out.push_str("/>\n");
            }
            BpmnNode::ServiceTask { id, name } => {
                out.push_str(&format!(r#"    <serviceTask id="{}""#, xml_escape(id)));
                if let Some(name) = name {
                    out.push_str(&format!(r#" name="{}""#, xml_escape(name)));
                }
                out.push_str("/>\n");
            }
            BpmnNode::ExclusiveGateway { id, name } => {
                out.push_str(&format!(r#"    <exclusiveGateway id="{}""#, xml_escape(id)));
                if let Some(name) = name {
                    out.push_str(&format!(r#" name="{}""#, xml_escape(name)));
                }
                out.push_str("/>\n");
            }
            BpmnNode::ParallelGateway { id, name } => {
                out.push_str(&format!(r#"    <parallelGateway id="{}""#, xml_escape(id)));
                if let Some(name) = name {
                    out.push_str(&format!(r#" name="{}""#, xml_escape(name)));
                }
                out.push_str("/>\n");
            }
            BpmnNode::RustType {
                id,
                name,
                kind,
                fields,
            } => {
                let kv: Vec<(String, String)> = fields
                    .iter()
                    .map(|f| (f.name.clone(), f.ty.clone()))
                    .collect();
                out.push_str(&format!(
                    r#"    <rustType id="{}" name="{}" kind="{}""#,
                    xml_escape(id),
                    xml_escape(name),
                    xml_escape(kind)
                ));
                if !kv.is_empty() {
                    out.push_str(&format!(r#" fields="{}""#, xml_escape(&kv_list_encode(&kv))));
                }
                out.push_str("/>\n");
            }
            BpmnNode::RustFunction {
                id,
                name,
                params,
                return_type,
                body,
            } => {
                let kv: Vec<(String, String)> = params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone()))
                    .collect();
                out.push_str(&format!(
                    r#"    <rustFunction id="{}" name="{}""#,
                    xml_escape(id),
                    xml_escape(name)
                ));
                if !kv.is_empty() {
                    out.push_str(&format!(r#" params="{}""#, xml_escape(&kv_list_encode(&kv))));
                }
                if let Some(rt) = return_type {
                    out.push_str(&format!(r#" returnType="{}""#, xml_escape(rt)));
                }
                if !body.is_empty() {
                    let joined = body.join("\n");
                    let body_hex = hex_encode(joined.as_bytes());
                    out.push_str(&format!(r#" bodyHex="{}""#, xml_escape(&body_hex)));
                }
                out.push_str("/>\n");
            }
            BpmnNode::Unknown { tag, id, name } => {
                out.push_str(&format!(r#"    <{}"#, xml_escape(tag)));
                if let Some(id) = id {
                    out.push_str(&format!(r#" id="{}""#, xml_escape(id)));
                }
                if let Some(name) = name {
                    out.push_str(&format!(r#" name="{}""#, xml_escape(name)));
                }
                out.push_str("/>\n");
            }
        }
    }

    for f in &proc.flows {
        out.push_str(&format!(
            r#"    <sequenceFlow id="{}" sourceRef="{}" targetRef="{}""#,
            xml_escape(&f.id),
            xml_escape(&f.source_ref),
            xml_escape(&f.target_ref)
        ));
        if let Some(name) = &f.name {
            out.push_str(&format!(r#" name="{}""#, xml_escape(name)));
        }
        out.push_str("/>\n");
    }

    out.push_str("  </process>\n</definitions>\n");
    out
}

pub fn find_start_event_id(proc: &BpmnProcess) -> Option<&str> {
    proc.nodes.iter().find_map(|n| match n {
        BpmnNode::StartEvent { id, .. } => Some(id.as_str()),
        _ => None,
    })
}

pub fn outgoing_map(proc: &BpmnProcess) -> HashMap<&str, Vec<&BpmnSequenceFlow>> {
    let mut outgoing: HashMap<&str, Vec<&BpmnSequenceFlow>> = HashMap::new();
    for f in &proc.flows {
        outgoing.entry(&f.source_ref).or_default().push(f);
    }
    for v in outgoing.values_mut() {
        v.sort_by(|a, b| a.id.cmp(&b.id));
    }
    outgoing
}

/// Best-effort: produce a single linear traversal by following the first
/// outgoing sequenceFlow from the start event.
pub fn best_effort_linearize(proc: &BpmnProcess) -> Vec<String> {
    let outgoing = outgoing_map(proc);
    let Some(mut cur) = find_start_event_id(proc) else {
        return Vec::new();
    };

    let mut seen: HashSet<&str> = HashSet::new();
    let mut order = Vec::new();

    loop {
        if !seen.insert(cur) {
            break;
        }
        let outs = outgoing.get(cur).cloned().unwrap_or_default();
        if outs.is_empty() {
            break;
        }
        let next = outs[0].target_ref.as_str();
        order.push(next.to_string());
        cur = next;
    }

    order
}
