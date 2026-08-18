-- OAuth 2.1 + PKCE authorization server for MCP clients (Claude, etc.):
-- dynamically registered clients, short-lived one-time authorization codes,
-- long-lived revocable refresh tokens. Access tokens are the existing JWT
-- (Claims in application/dto/auth.rs) and are never stored server-side.

CREATE TABLE oauth_clients (
    client_id TEXT PRIMARY KEY,
    client_name TEXT NOT NULL,
    redirect_uris TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE oauth_authorization_codes (
    code TEXT PRIMARY KEY,
    client_id TEXT NOT NULL REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES utilisateurs(id) ON DELETE CASCADE,
    redirect_uri TEXT NOT NULL,
    -- PKCE (RFC 7636): S256 only, "plain" is not accepted (OAuth 2.1).
    code_challenge TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_oauth_codes_expires ON oauth_authorization_codes (expires_at);

CREATE TABLE oauth_refresh_tokens (
    -- SHA-256 of the actual token: the token itself is a bearer secret and is
    -- never persisted, same principle as a password hash.
    token_hash TEXT PRIMARY KEY,
    client_id TEXT NOT NULL REFERENCES oauth_clients(client_id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES utilisateurs(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_oauth_refresh_tokens_user ON oauth_refresh_tokens (user_id);
