const API_URL = import.meta.env.VITE_API_URL;

/** Thrown on any non-2xx response, carrying the status and server message. */
export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
    this.name = "ApiError";
  }
}

const TOKEN_KEY = "kalshi_token";

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}
export function setToken(token: string) {
  localStorage.setItem(TOKEN_KEY, token);
}
export function clearToken() {
  localStorage.removeItem(TOKEN_KEY);
}

interface RequestOptions {
  method?: "GET" | "POST" | "DELETE";
  body?: unknown;
  auth?: boolean; // attach the bearer token? default true
}

/** Run the fetch, normalize errors to ApiError, and hand back the raw text. */
async function request(path: string, opts: RequestOptions): Promise<string> {
  const { method = "GET", body, auth = true } = opts;

  const headers: Record<string, string> = {};
  if (body !== undefined) headers["Content-Type"] = "application/json";
  if (auth) {
    const token = getToken();
    if (token) headers["Authorization"] = `Bearer ${token}`;
  }

  const res = await fetch(`${API_URL}${path}`, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });

  const text = await res.text();

  if (!res.ok) {
    // Backend always sends {"error": "..."} on failure (Chapter 1).
    let message = `request failed (${res.status})`;
    try {
      const data = text ? JSON.parse(text) : null;
      if (data?.error) message = data.error;
    } catch {
      // non-JSON error body — keep the generic message
    }
    throw new ApiError(res.status, message);
  }

  return text;
}

/** The single entry point for JSON endpoints. Generic over the response type. */
export async function api<T>(path: string, opts: RequestOptions = {}): Promise<T> {
  const text = await request(path, opts);
  // 204 No Content or empty body: nothing to parse.
  const data = text ? JSON.parse(text) : null;
  return data as T;
}

/** For endpoints that return a raw (non-JSON) text body — e.g. login returns
 *  the bare JWT. Same auth/error handling, no JSON.parse on success. */
export async function apiText(path: string, opts: RequestOptions = {}): Promise<string> {
  return request(path, opts);
}
