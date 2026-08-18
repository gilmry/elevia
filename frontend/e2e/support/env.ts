/**
 * The dev stack bootstraps its one hardcoded account from these two env vars
 * (see backend `ADMIN_EMAIL`/`ADMIN_PASSWORD`, defaulted in .env.example).
 * Override via the same names if your local .env diverges.
 */
export const ADMIN_EMAIL = process.env.ADMIN_EMAIL ?? "admin@elevia.local";
export const ADMIN_PASSWORD = process.env.ADMIN_PASSWORD ?? "change-me-immediately";

/**
 * There is no reset endpoint and the dev Postgres persists across runs, so
 * every spec that creates data (exploitations, products) must use names
 * unique to that run to stay independent of what earlier runs left behind.
 */
export function unique(label: string): string {
  const stamp = `${Date.now()}-${Math.floor(Math.random() * 1000)}`;
  return `${label}-${stamp}`;
}
