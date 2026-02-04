type InvokeFn = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

type E2EState = {
  backendUrl?: string;
  files?: Record<string, string>;
  dialog?: {
    openQueue?: Array<string | null>;
    saveQueue?: Array<string | null>;
  };
  invoke?: {
    bpmn_to_rust?: (xml: string) => string;
    rust_to_ts?: (rust: string) => string;
    rust_to_bpmn?: (rust: string) => string;
    ts_to_bpmn?: (ts: string) => string;
    validate_roundtrip?: (xml: string) => unknown;
  };
};

declare global {
  // eslint-disable-next-line no-var
  var __RUST_TO_TS_E2E__: E2EState | undefined;
}

function getE2E(): E2EState | undefined {
  return (globalThis as any).__RUST_TO_TS_E2E__;
}

function isTauriRuntime(): boolean {
  const w = globalThis as any;
  return Boolean(w?.__TAURI_INTERNALS__ || w?.__TAURI__);
}

function backendUrl(): string | null {
  try {
    const env = (import.meta as any)?.env;
    const v = env?.VITE_RUST_BACKEND_URL;
    if (typeof v === 'string' && v.length) return v;
  } catch {
    // fall through
  }

  // In E2E (browser tests), allow the test harness to specify the backend URL
  // without relying on Vite env injection, which can be finicky on Windows.
  const e2e = getE2E();
  if (e2e?.backendUrl && typeof e2e.backendUrl === 'string' && e2e.backendUrl.length) {
    return e2e.backendUrl;
  }
  if (e2e) {
    return 'http://127.0.0.1:15123';
  }

  return null;
}

async function invokeViaHttp<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const base = backendUrl();
  if (!base) throw new Error('No VITE_RUST_BACKEND_URL configured');
  const url = `${base.replace(/\/$/, '')}/invoke/${encodeURIComponent(command)}`;
  const res = await fetch(url, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(args ?? {})
  });
  const body = await res.json().catch(() => null);
  if (!res.ok) {
    const msg = body?.error ? String(body.error) : `HTTP ${res.status}`;
    throw new Error(msg);
  }
  if (!body || body.ok !== true) {
    throw new Error(body?.error ? String(body.error) : 'Backend returned error');
  }
  return body.result as T;
}

let invokeImpl: InvokeFn | null = null;

export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (invokeImpl) return invokeImpl<T>(command, args);

  if (isTauriRuntime()) {
    const mod = await import('@tauri-apps/api/core');
    invokeImpl = mod.invoke as InvokeFn;
    return invokeImpl<T>(command, args);
  }

  invokeImpl = async <U>(cmd: string, a?: Record<string, unknown>) => {
    const e2e = getE2E();
    // If a real backend is configured, prefer it unless the test explicitly overrides the command.
    const hasBackend = Boolean(backendUrl());
    const override = e2e?.invoke as any;
    if (hasBackend && !(override && typeof override[cmd] === 'function')) {
      return await invokeViaHttp<U>(cmd, a);
    }

    if (!e2e) {
      throw new Error(
        `invoke(${cmd}) is not available outside Tauri (set VITE_RUST_BACKEND_URL or __RUST_TO_TS_E2E__)`
      );
    }

    if (cmd === 'read_text_file') {
      const p = String((a as any)?.path ?? '');
      const v = e2e.files?.[p];
      if (typeof v !== 'string') throw new Error(`E2E: missing file: ${p}`);
      return v as unknown as U;
    }

    if (cmd === 'write_text_file') {
      const p = String((a as any)?.path ?? '');
      const contents = String((a as any)?.contents ?? '');
      e2e.files = e2e.files ?? {};
      e2e.files[p] = contents;
      return undefined as unknown as U;
    }

    if (cmd === 'bpmn_to_rust') {
      const xml = String((a as any)?.xml ?? '');
      const f = e2e.invoke?.bpmn_to_rust;
      return (f ? f(xml) : `// e2e stub\nfn main() {\n    println!(\"stub\");\n}\n`) as unknown as U;
    }

    if (cmd === 'rust_to_ts') {
      const rust = String((a as any)?.rust ?? '');
      const f = e2e.invoke?.rust_to_ts;
      return (f ? f(rust) : `export function main(): void {\n  console.log(\"stub\");\n}\n`) as unknown as U;
    }

    if (cmd === 'rust_to_bpmn') {
      const rust = String((a as any)?.rust ?? '');
      const f = e2e.invoke?.rust_to_bpmn;
      return (f ? f(rust) : String(e2e.files?.[String((e2e.dialog?.openQueue?.[0] ?? '') as any)] ?? '')) as unknown as U;
    }

    if (cmd === 'ts_to_bpmn') {
      const ts = String((a as any)?.ts ?? '');
      const f = e2e.invoke?.ts_to_bpmn;
      return (f ? f(ts) : String(e2e.files?.[String((e2e.dialog?.openQueue?.[0] ?? '') as any)] ?? '')) as unknown as U;
    }

    if (cmd === 'validate_roundtrip') {
      const xml = String((a as any)?.xml ?? '');
      const f = e2e.invoke?.validate_roundtrip;
      return (f ? f(xml) : {
        ok: true,
        stdout_direct: 'ok',
        stdout_roundtrip: 'ok',
        rust_direct: 'ok',
        rust_roundtrip: 'ok',
        bpmn_roundtrip: xml
      }) as unknown as U;
    }

    throw new Error(`E2E: invoke(${cmd}) not implemented`);
  };

  return invokeImpl<T>(command, args);
}

export async function open(opts: unknown): Promise<string | string[] | null> {
  if (isTauriRuntime()) {
    const mod = await import('@tauri-apps/plugin-dialog');
    return await mod.open(opts as any);
  }
  const e2e = getE2E();
  if (!e2e) return null;
  const v = e2e.dialog?.openQueue?.length ? e2e.dialog.openQueue.shift()! : null;
  return v;
}

export async function save(opts: unknown): Promise<string | null> {
  if (isTauriRuntime()) {
    const mod = await import('@tauri-apps/plugin-dialog');
    return await mod.save(opts as any);
  }
  const e2e = getE2E();
  if (!e2e) return null;
  const v = e2e.dialog?.saveQueue?.length ? e2e.dialog.saveQueue.shift()! : null;
  return v;
}
