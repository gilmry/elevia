//! OAuth 2.1 + PKCE authorization server, so an MCP client (Claude, etc.) can
//! offer a real "Connect" button instead of asking the user to paste a JWT
//! that expires in 12h. Fully self-hosted: no external identity provider,
//! delegates the actual credential check to the same email/password
//! `utilisateurs` table the REST API already uses.
//!
//! Access tokens minted here are the exact same JWT shape as `/auth/login`
//! (see `AuthUseCases::mint_token`), just shorter-lived (1h, refreshable) -
//! `/mcp` and every REST handler need no changes to accept either.

use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

use crate::application::dto::{
    AuthorizeFormDto, AuthorizeParams, RegisterClientDto, TokenRequestDto,
};
use crate::application::use_cases::OAuthError;
use crate::domain::entities::OAuthClient;
use crate::infrastructure::web::app_state::AppState;

fn bad_request(message: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({ "error": message }))
}

/// `PUBLIC_API_URL` (already used by the frontend build to reach the API) if
/// set, otherwise derived from the request. `connection_info()` trusts
/// `X-Forwarded-*` headers unconditionally, which is fine behind Traefik (the
/// backend has no published port in prod - see docker-compose.yml) but not a
/// value we want an MCP client treating as a verified server identity when a
/// stable one is available for free from config.
fn base_url(req: &HttpRequest) -> String {
    if let Ok(url) = std::env::var("PUBLIC_API_URL") {
        return url.trim_end_matches('/').to_string();
    }
    let info = req.connection_info();
    format!("{}://{}", info.scheme(), info.host())
}

pub async fn metadata(req: HttpRequest) -> HttpResponse {
    let issuer = base_url(&req);
    HttpResponse::Ok().json(json!({
        "issuer": issuer,
        "authorization_endpoint": format!("{issuer}/oauth/authorize"),
        "token_endpoint": format!("{issuer}/oauth/token"),
        "registration_endpoint": format!("{issuer}/oauth/register"),
        "scopes_supported": ["elevia"],
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none"],
    }))
}

pub async fn register(
    state: web::Data<AppState>,
    dto: web::Json<RegisterClientDto>,
) -> HttpResponse {
    match state.oauth_use_cases.register_client(dto.into_inner()).await {
        Ok(resp) => HttpResponse::Created().json(resp),
        Err(OAuthError::InvalidRedirectUris) => bad_request(
            "redirect_uris must be a non-empty list of absolute http(s) URIs (http only for localhost)",
        ),
        Err(err) => {
            tracing::error!(?err, "oauth register_client failed");
            HttpResponse::InternalServerError().json(json!({ "error": "internal error" }))
        }
    }
}

pub async fn authorize_get(
    state: web::Data<AppState>,
    query: web::Query<AuthorizeParams>,
) -> HttpResponse {
    let params = query.into_inner();
    match state
        .oauth_use_cases
        .validate_authorize_request(
            &params.client_id,
            &params.redirect_uri,
            &params.response_type,
            &params.code_challenge_method,
        )
        .await
    {
        Ok(client) => html_response(render_form(&client, &params, None)),
        Err(err) => html_error(&err),
    }
}

pub async fn authorize_post(
    state: web::Data<AppState>,
    form: web::Form<AuthorizeFormDto>,
) -> HttpResponse {
    let dto = form.into_inner();

    match state.oauth_use_cases.authorize(&dto).await {
        Ok(code) => {
            let redirect_uri = &dto.redirect_uri;
            let sep = if redirect_uri.contains('?') { '&' } else { '?' };
            let location = format!(
                "{redirect_uri}{sep}code={}&state={}",
                urlencoding::encode(&code),
                urlencoding::encode(&dto.state),
            );
            HttpResponse::Found()
                .append_header(("Location", location))
                .finish()
        }
        Err(OAuthError::InvalidCredentials) => {
            let params = AuthorizeParams {
                client_id: dto.client_id.clone(),
                redirect_uri: dto.redirect_uri.clone(),
                response_type: "code".to_string(),
                state: dto.state.clone(),
                code_challenge: dto.code_challenge.clone(),
                code_challenge_method: dto.code_challenge_method.clone(),
            };
            match state
                .oauth_use_cases
                .validate_authorize_request(
                    &params.client_id,
                    &params.redirect_uri,
                    &params.response_type,
                    &params.code_challenge_method,
                )
                .await
            {
                Ok(client) => html_response(render_form(
                    &client,
                    &params,
                    Some("Email ou mot de passe incorrect."),
                )),
                Err(err) => html_error(&err),
            }
        }
        Err(err) => html_error(&err),
    }
}

pub async fn token(state: web::Data<AppState>, form: web::Form<TokenRequestDto>) -> HttpResponse {
    match state.oauth_use_cases.token(form.into_inner()).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(err @ OAuthError::UnsupportedGrantType) => {
            oauth_token_error("unsupported_grant_type", &err)
        }
        Err(
            err @ (OAuthError::InvalidGrant
            | OAuthError::InvalidPkce
            | OAuthError::InvalidRefreshToken),
        ) => oauth_token_error("invalid_grant", &err),
        Err(err @ OAuthError::UnknownClient) => oauth_token_error("invalid_client", &err),
        Err(err) => {
            tracing::error!(?err, "oauth token exchange failed");
            oauth_token_error("server_error", &err)
        }
    }
}

/// RFC 6749 §5.2 error shape for the token endpoint.
fn oauth_token_error(error: &str, err: &OAuthError) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({
        "error": error,
        "error_description": err.to_string(),
    }))
}

fn html_response(body: String) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}

fn html_error(err: &OAuthError) -> HttpResponse {
    HttpResponse::BadRequest()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            "<!doctype html><html><body><p>Erreur : {}</p></body></html>",
            html_escape(&err.to_string())
        ))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_form(client: &OAuthClient, params: &AuthorizeParams, error: Option<&str>) -> String {
    let error_html = error
        .map(|msg| format!("<p style=\"color:#c62828\">{}</p>", html_escape(msg)))
        .unwrap_or_default();

    format!(
        r#"<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<title>Connexion Elevia</title>
<style>
  body {{ font-family: system-ui, sans-serif; max-width: 24rem; margin: 3rem auto; padding: 0 1rem; }}
  label {{ display: block; margin-bottom: 1rem; }}
  input {{ display: block; width: 100%; padding: 0.5rem; margin-top: 0.25rem; box-sizing: border-box; }}
  button {{ width: 100%; padding: 0.6rem; }}
</style>
</head>
<body>
<h1>Elevia</h1>
<p><strong>{client_name}</strong> demande à accéder à votre compte Elevia.</p>
{error_html}
<form method="post" action="/oauth/authorize">
  <label>Email<input type="email" name="email" required autocomplete="username"></label>
  <label>Mot de passe<input type="password" name="password" required autocomplete="current-password"></label>
  <input type="hidden" name="client_id" value="{client_id}">
  <input type="hidden" name="redirect_uri" value="{redirect_uri}">
  <input type="hidden" name="state" value="{state}">
  <input type="hidden" name="code_challenge" value="{code_challenge}">
  <input type="hidden" name="code_challenge_method" value="{code_challenge_method}">
  <button type="submit">Autoriser</button>
</form>
</body>
</html>"#,
        client_name = html_escape(&client.client_name),
        error_html = error_html,
        client_id = html_escape(&params.client_id),
        redirect_uri = html_escape(&params.redirect_uri),
        state = html_escape(&params.state),
        code_challenge = html_escape(&params.code_challenge),
        code_challenge_method = html_escape(&params.code_challenge_method),
    )
}
