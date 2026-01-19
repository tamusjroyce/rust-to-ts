import { extname, join } from "https://deno.land/std@0.224.0/path/mod.ts";

const ROOT = new URL("./public/", import.meta.url);

const MIME: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".ts": "text/javascript; charset=utf-8",
  ".map": "application/json; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".bpmn": "application/xml; charset=utf-8",
  ".xml": "application/xml; charset=utf-8",
};

function contentType(path: string): string {
  return MIME[extname(path).toLowerCase()] ?? "application/octet-stream";
}

function safePath(urlPath: string): string {
  // Prevent path traversal
  const cleaned = urlPath.replaceAll("\\", "/");
  const parts = cleaned.split("/").filter((p) => p && p !== "." && p !== "..");
  return "/" + parts.join("/");
}

Deno.serve({ port: 8000 }, async (req) => {
  const url = new URL(req.url);
  const path = safePath(url.pathname);

  // Default doc
  const rel = path === "/" ? "/index.html" : path;

  const fileUrl = new URL("." + rel, ROOT);

  try {
    const file = await Deno.open(fileUrl, { read: true });
    return new Response(file.readable, {
      headers: {
        "content-type": contentType(rel),
        // allow esm.sh to be fetched in browser
        "cache-control": "no-store",
      },
    });
  } catch {
    return new Response("Not found", { status: 404 });
  }
});
