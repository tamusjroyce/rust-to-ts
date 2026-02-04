use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use axum::{
    extract::State,
    http::{header, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use rust_to_ts::ast_v2::bpmn::{convert_bpmn_xml_to_rust_code, convert_rust_code_to_bpmn_xml};
use rust_to_ts::ast_v2::convert_ts_file_to_rust;
use rust_to_ts::converter::convert_rust_src_to_ts;

#[derive(Clone)]
struct AppState {
    tmp_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ReadTextFileReq {
    path: String,
}

#[derive(Debug, Deserialize)]
struct WriteTextFileReq {
    path: String,
    contents: String,
}

#[derive(Debug, Deserialize)]
struct XmlReq {
    xml: String,
}

#[derive(Debug, Deserialize)]
struct RustReq {
    rust: String,
}

#[derive(Debug, Deserialize)]
struct TsReq {
    ts: String,
}

#[derive(Debug, Serialize)]
struct ApiOk<T> {
    ok: bool,
    result: T,
}

#[derive(Debug, Serialize)]
struct ApiErr {
    ok: bool,
    error: String,
}

fn ok<T: Serialize>(value: T) -> Response {
    (StatusCode::OK, Json(ApiOk { ok: true, result: value })).into_response()
}

fn err(status: StatusCode, message: String) -> Response {
    (status, Json(ApiErr { ok: false, error: message })).into_response()
}

fn tmp_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("rust-to-ts-bpmn-editor");
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create temp dir {}: {e}", dir.display()))?;
    Ok(dir)
}

fn exe_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", base)
    } else {
        base.to_string()
    }
}

fn compile_rust_to_exe(src_path: &PathBuf, exe_path: &PathBuf) -> Result<(), String> {
    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg(src_path)
        .arg("-o")
        .arg(exe_path)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run rustc: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "rustc failed\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn run_exe(exe_path: &PathBuf) -> Result<String, String> {
    let output = Command::new(exe_path)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run {}: {e}", exe_path.display()))?;

    if !output.status.success() {
        return Err(format!(
            "Program exited non-zero\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[derive(Debug, Serialize)]
struct ValidateResult {
    ok: bool,
    stdout_direct: String,
    stdout_roundtrip: String,
    rust_direct: String,
    rust_roundtrip: String,
    bpmn_roundtrip: String,
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn read_text_file(Json(req): Json<ReadTextFileReq>) -> impl IntoResponse {
    match fs::read_to_string(&req.path) {
        Ok(s) => ok(s),
        Err(e) => err(
            StatusCode::BAD_REQUEST,
            format!("Failed to read {}: {e}", req.path),
        ),
    }
}

async fn write_text_file(Json(req): Json<WriteTextFileReq>) -> impl IntoResponse {
    match fs::write(&req.path, req.contents) {
        Ok(()) => ok(true),
        Err(e) => err(
            StatusCode::BAD_REQUEST,
            format!("Failed to write {}: {e}", req.path),
        ),
    }
}

async fn bpmn_to_rust(Json(req): Json<XmlReq>) -> impl IntoResponse {
    match convert_bpmn_xml_to_rust_code(&req.xml) {
        Ok(s) => ok(s),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

async fn bpmn_to_ts(Json(req): Json<XmlReq>) -> impl IntoResponse {
    match convert_bpmn_xml_to_rust_code(&req.xml)
        .and_then(|rust| convert_rust_src_to_ts(&rust, false))
    {
        Ok(s) => ok(s),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

async fn rust_to_ts(Json(req): Json<RustReq>) -> impl IntoResponse {
    match convert_rust_src_to_ts(&req.rust, false) {
        Ok(s) => ok(s),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

async fn rust_to_bpmn(Json(req): Json<RustReq>) -> impl IntoResponse {
    match convert_rust_code_to_bpmn_xml(&req.rust) {
        Ok(s) => ok(s),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

async fn ts_to_bpmn(State(state): State<AppState>, Json(req): Json<TsReq>) -> impl IntoResponse {
    let ts_path = state.tmp_dir.join("sync_from_ts.ts");
    if let Err(e) = fs::write(&ts_path, &req.ts) {
        return err(
            StatusCode::BAD_REQUEST,
            format!("Failed to write {}: {e}", ts_path.display()),
        );
    }

    match convert_ts_file_to_rust(&ts_path).and_then(|rust| convert_rust_code_to_bpmn_xml(&rust)) {
        Ok(s) => ok(s),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

async fn validate_roundtrip(State(state): State<AppState>, Json(req): Json<XmlReq>) -> impl IntoResponse {
    // Direct
    let rust_direct = match convert_bpmn_xml_to_rust_code(&req.xml) {
        Ok(s) => s,
        Err(e) => return err(StatusCode::BAD_REQUEST, e),
    };

    // Round-trip through BPMN
    let bpmn_roundtrip = match convert_rust_code_to_bpmn_xml(&rust_direct) {
        Ok(s) => s,
        Err(e) => return err(StatusCode::BAD_REQUEST, e),
    };
    let rust_roundtrip = match convert_bpmn_xml_to_rust_code(&bpmn_roundtrip) {
        Ok(s) => s,
        Err(e) => return err(StatusCode::BAD_REQUEST, e),
    };

    let direct_rs = state.tmp_dir.join("direct.rs");
    let round_rs = state.tmp_dir.join("roundtrip.rs");
    let direct_exe = state.tmp_dir.join(exe_name("direct"));
    let round_exe = state.tmp_dir.join(exe_name("roundtrip"));

    if let Err(e) = fs::write(&direct_rs, &rust_direct) {
        return err(
            StatusCode::BAD_REQUEST,
            format!("Failed to write {}: {e}", direct_rs.display()),
        );
    }
    if let Err(e) = fs::write(&round_rs, &rust_roundtrip) {
        return err(
            StatusCode::BAD_REQUEST,
            format!("Failed to write {}: {e}", round_rs.display()),
        );
    }

    if let Err(e) = compile_rust_to_exe(&direct_rs, &direct_exe) {
        return err(StatusCode::BAD_REQUEST, e);
    }
    if let Err(e) = compile_rust_to_exe(&round_rs, &round_exe) {
        return err(StatusCode::BAD_REQUEST, e);
    }

    let stdout_direct = match run_exe(&direct_exe) {
        Ok(s) => s,
        Err(e) => return err(StatusCode::BAD_REQUEST, e),
    };
    let stdout_roundtrip = match run_exe(&round_exe) {
        Ok(s) => s,
        Err(e) => return err(StatusCode::BAD_REQUEST, e),
    };

    let ok_match = stdout_direct == stdout_roundtrip;

    ok(ValidateResult {
        ok: ok_match,
        stdout_direct,
        stdout_roundtrip,
        rust_direct,
        rust_roundtrip,
        bpmn_roundtrip,
    })
}

#[derive(Debug)]
struct Args {
    port: u16,
}

fn parse_args() -> Result<Args, String> {
    let mut port: u16 = 15123;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        if a == "--port" {
            let v = it.next().ok_or_else(|| "--port requires a value".to_string())?;
            port = v
                .parse::<u16>()
                .map_err(|_| format!("Invalid --port value: {v}"))?;
        } else {
            return Err(format!("Unknown arg: {a}"));
        }
    }
    Ok(Args { port })
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = parse_args()?;
    let state = AppState { tmp_dir: tmp_dir()? };

    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("*"))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/health", get(health))
        .route("/invoke/read_text_file", post(read_text_file))
        .route("/invoke/write_text_file", post(write_text_file))
        .route("/invoke/bpmn_to_rust", post(bpmn_to_rust))
        .route("/invoke/bpmn_to_ts", post(bpmn_to_ts))
        .route("/invoke/rust_to_ts", post(rust_to_ts))
        .route("/invoke/rust_to_bpmn", post(rust_to_bpmn))
        .route("/invoke/ts_to_bpmn", post(ts_to_bpmn))
        .route("/invoke/validate_roundtrip", post(validate_roundtrip))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    println!("[bpmn-editor-backend] Listening on http://{addr}");

    axum::serve(tokio::net::TcpListener::bind(addr).await.map_err(|e| e.to_string())?, app)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
