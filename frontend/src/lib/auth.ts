import type { AuthClaims } from "./types";

type Role = AuthClaims["role"];

const TOKEN_KEY = "elevia_token";

export function saveToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function clearToken(): void {
  localStorage.removeItem(TOKEN_KEY);
}

function decodeClaims(token: string): AuthClaims | null {
  try {
    const payload = token.split(".")[1];
    const base64 = payload.replace(/-/g, "+").replace(/_/g, "/");
    return JSON.parse(atob(base64));
  } catch {
    return null;
  }
}

/** The current user's claims, or `null` if not logged in / the token expired. */
export function getCurrentUser(): AuthClaims | null {
  const token = getToken();
  if (!token) return null;

  const claims = decodeClaims(token);
  if (!claims) return null;

  if (claims.exp * 1000 < Date.now()) {
    clearToken();
    return null;
  }

  return claims;
}

export function logout(): void {
  clearToken();
  window.location.href = "/login";
}

/**
 * Redirects to /login if not authenticated (or to `/` if authenticated but the
 * wrong role for this page). Call from a Svelte component's `onMount`. Returns
 * the current user's claims so the caller can render, or `null` if it redirected.
 */
export function requireAuth(role?: Role): AuthClaims | null {
  const user = getCurrentUser();
  if (!user) {
    window.location.href = "/login";
    return null;
  }
  if (role && user.role !== role) {
    window.location.href = "/";
    return null;
  }
  return user;
}
