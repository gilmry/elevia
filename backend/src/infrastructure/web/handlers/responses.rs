use actix_web::HttpResponse;
use serde_json::json;

pub fn bad_request(message: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({ "error": message }))
}

pub fn forbidden(message: &str) -> HttpResponse {
    HttpResponse::Forbidden().json(json!({ "error": message }))
}

pub fn unauthorized(message: &str) -> HttpResponse {
    HttpResponse::Unauthorized().json(json!({ "error": message }))
}

pub fn internal_error() -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({ "error": "internal error" }))
}

pub fn not_found(message: &str) -> HttpResponse {
    HttpResponse::NotFound().json(json!({ "error": message }))
}

pub const FORBIDDEN_EXPLOITATION: &str = "access denied: resource belongs to another exploitation";
pub const FORBIDDEN_ADMIN: &str = "admin role required";
