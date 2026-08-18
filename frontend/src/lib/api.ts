import { clearToken, getToken } from "./auth";
import type {
  CoopDashboard,
  CreateEntryInput,
  CreateExploitationInput,
  CreateProductInput,
  CreateProductionInput,
  Entry,
  Exploitation,
  ExploitationDashboard,
  ExploitationStatus,
  Product,
  Production,
  UpdateProductInput,
} from "./types";

const API_URL = import.meta.env.PUBLIC_API_URL ?? "http://localhost:8080";

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

/** Thrown by write operations when the browser has no network connection at all. */
export class OfflineError extends Error {
  constructor() {
    super("no network connection");
  }
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string> | undefined),
  };
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  let response: Response;
  try {
    response = await fetch(`${API_URL}${path}`, { ...options, headers });
  } catch {
    throw new OfflineError();
  }

  if (response.status === 401) {
    clearToken();
  }

  if (!response.ok) {
    let message = `la requête a échoué (${response.status})`;
    try {
      const body = await response.json();
      if (body?.error) message = body.error;
    } catch {
      // response body wasn't JSON - keep the generic message
    }
    throw new ApiError(response.status, message);
  }

  if (response.status === 204) {
    return undefined as T;
  }
  return response.json();
}

export const api = {
  login: (email: string, password: string) =>
    request<{ token: string }>("/auth/login", {
      method: "POST",
      body: JSON.stringify({ email, password }),
    }),

  listProducts: () => request<Product[]>("/products"),

  listEntries: (exploitationId: string) =>
    request<Entry[]>(`/exploitations/${exploitationId}/entries`),

  submitEntry: (exploitationId: string, input: CreateEntryInput) =>
    request<Entry>(`/exploitations/${exploitationId}/entries`, {
      method: "POST",
      body: JSON.stringify(input),
    }),

  submitProduction: (exploitationId: string, input: CreateProductionInput) =>
    request<Production>(`/exploitations/${exploitationId}/production`, {
      method: "POST",
      body: JSON.stringify(input),
    }),

  getDashboard: (exploitationId: string) =>
    request<ExploitationDashboard>(`/exploitations/${exploitationId}/dashboard`),

  getCoopDashboard: () => request<CoopDashboard>("/coop/dashboard"),

  createExploitation: (input: CreateExploitationInput) =>
    request<Exploitation>("/admin/exploitations", {
      method: "POST",
      body: JSON.stringify(input),
    }),

  listExploitations: () => request<ExploitationStatus[]>("/admin/exploitations"),

  createProduct: (input: CreateProductInput) =>
    request<Product>("/admin/products", {
      method: "POST",
      body: JSON.stringify(input),
    }),

  updateProduct: (id: string, input: UpdateProductInput) =>
    request<Product>(`/admin/products/${id}`, {
      method: "PUT",
      body: JSON.stringify(input),
    }),
};
