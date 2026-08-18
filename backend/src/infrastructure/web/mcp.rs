//! MCP (Model Context Protocol) endpoint: lets an admin or exploitation user
//! plug Claude directly into their Elevia account as a "remote MCP server".
//!
//! Single `POST /mcp` endpoint speaking JSON-RPC 2.0 over the MCP "Streamable
//! HTTP" transport, stateless (every request re-authenticates via the same
//! `Authorization: Bearer <JWT>` header the REST API already uses - no
//! separate session/OAuth flow). Read-only for v1: every tool reuses an
//! existing use case exactly as the REST handlers do, so there is no new
//! business logic here, only a JSON-RPC adapter and a role-based tool list.

use actix_web::{error, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::middleware::AuthenticatedUser;

const PROTOCOL_VERSION: &str = "2025-06-18";

/// Malformed request bodies would otherwise fall through to actix's default
/// plain-text 400, which isn't valid JSON-RPC and could confuse a strict MCP
/// client. `id` is unknown at this point (the body didn't even parse), so it
/// is null per the JSON-RPC spec for that case.
pub fn json_error_handler(err: error::JsonPayloadError, _req: &HttpRequest) -> error::Error {
    let body = json!({
        "jsonrpc": "2.0",
        "id": Value::Null,
        "error": { "code": -32700, "message": format!("parse error: {err}") },
    });
    error::InternalError::from_response(err, HttpResponse::BadRequest().json(body)).into()
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcErrorObj>,
}

#[derive(Debug, Serialize)]
struct JsonRpcErrorObj {
    code: i32,
    message: String,
}

impl JsonRpcResponse {
    fn result(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcErrorObj {
                code,
                message: message.into(),
            }),
        }
    }
}

pub async fn handle(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    body: web::Json<JsonRpcRequest>,
) -> HttpResponse {
    let req = body.into_inner();

    // A request with no `id` is a JSON-RPC notification: no response body.
    // The only one MCP clients send here is `notifications/initialized`.
    let Some(id) = req.id else {
        return HttpResponse::Accepted().finish();
    };

    let response = match req.method.as_str() {
        "initialize" => JsonRpcResponse::result(id, initialize_result()),
        "tools/list" => JsonRpcResponse::result(id, json!({ "tools": tool_schemas(&user) })),
        "tools/call" => JsonRpcResponse::result(id, call_tool(&state, &user, req.params).await),
        other => JsonRpcResponse::error(id, -32601, format!("method not found: {other}")),
    };

    HttpResponse::Ok().json(response)
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "elevia-mcp", "version": env!("CARGO_PKG_VERSION") },
    })
}

fn tool(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": { "type": "object", "properties": {} },
    })
}

/// Tools visible to this user - `tools/call` re-checks the same role
/// requirement, so this list controls discoverability, not authorization.
fn tool_schemas(user: &AuthenticatedUser) -> Vec<Value> {
    let mut tools = vec![
        tool(
            "list_products",
            "Liste le catalogue des produits/intrants Elevia (nom, unité, catégorie).",
        ),
        tool(
            "get_coop_dashboard",
            "Vue agrégée de la coopérative pour le mois courant : besoins en intrants, \
             marge moyenne, quartiles de coût par unité. N'expose jamais les chiffres \
             nominaux d'une exploitation individuelle.",
        ),
    ];

    if user.role == "exploitation" {
        tools.push(tool(
            "list_my_entries",
            "Liste les coûts (intrants) saisis par mon exploitation, tous mois confondus.",
        ));
        tools.push(tool(
            "get_my_dashboard",
            "Mon dashboard mensuel : coût total, quantité produite, coût par unité, \
             marge estimée, mois par mois.",
        ));
    }

    if user.is_admin() {
        tools.push(tool(
            "list_exploitations",
            "Liste toutes les exploitations membres avec leur statut de saisie \
             (coûts/production) pour le mois courant.",
        ));
    }

    tools
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
}

async fn call_tool(state: &AppState, user: &AuthenticatedUser, params: Value) -> Value {
    let params: ToolCallParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(_) => return error_result("invalid tool call params: expected { name, arguments }"),
    };

    let outcome = match params.name.as_str() {
        "list_products" => run(
            state.catalog_use_cases.list_products().await,
            "list_products",
        ),
        "get_coop_dashboard" => run(
            state.coop_use_cases.coop_dashboard().await,
            "get_coop_dashboard",
        ),
        "list_my_entries" => match own_exploitation_id(user) {
            Ok(id) => run(
                state.entry_use_cases.list_entries(id).await,
                "list_my_entries",
            ),
            Err(msg) => Err(msg),
        },
        "get_my_dashboard" => match own_exploitation_id(user) {
            Ok(id) => run(
                state.dashboard_use_cases.exploitation_dashboard(id).await,
                "get_my_dashboard",
            ),
            Err(msg) => Err(msg),
        },
        "list_exploitations" => {
            if user.is_admin() {
                run(
                    state.admin_use_cases.list_exploitations_with_status().await,
                    "list_exploitations",
                )
            } else {
                Err("admin role required".to_string())
            }
        }
        other => Err(format!("unknown tool: {other}")),
    };

    match outcome {
        Ok(value) => success_result(&value),
        Err(message) => error_result(&message),
    }
}

fn own_exploitation_id(user: &AuthenticatedUser) -> Result<uuid::Uuid, String> {
    user.exploitation_id
        .ok_or_else(|| "this tool is reserved to exploitation accounts".to_string())
}

fn run<T: Serialize, E: std::fmt::Debug>(
    result: Result<T, E>,
    tool_name: &str,
) -> Result<Value, String> {
    match result {
        Ok(value) => serde_json::to_value(value).map_err(|_| "serialization error".to_string()),
        Err(err) => {
            tracing::error!(?err, tool_name, "mcp tool call failed");
            Err("internal error".to_string())
        }
    }
}

fn success_result(value: &Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": value.to_string() }],
        "isError": false,
    })
}

fn error_result(message: &str) -> Value {
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true,
    })
}
