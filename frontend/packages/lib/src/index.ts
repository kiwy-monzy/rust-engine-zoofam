/**
 * @gateway/lib — the single place that knows how to talk to the gateway.
 *
 * Auth model:
 *   - The browser SPA relies on the httpOnly `gw_access` / `gw_refresh`
 *     cookies set by the gateway at /auth/login. We send `credentials: include`
 *     so cookies travel with every request.
 *   - For non-browser clients (CLI, server-to-server) we keep an
 *     `Authorization: Bearer …` fallback. The localStorage copy is the
 *     mirror of the access token and is only used as a back-up.
 *   - A single-flight refresh interceptor retries 401s with the refresh
 *     cookie via /auth/refresh. If the refresh fails, the session is
 *     cleared exactly once and the global `onUnauthorized` fires so the
 *     React tree can navigate to /login.
 */

const API_BASE = "/api/v1";
export const TOKEN_KEY = "gateway.token";
export const USER_KEY = "gateway.user";
export const ROLES_KEY = "gateway.roles";
export const PERMS_KEY = "gateway.perms";

let refreshInflight: Promise<boolean> | null = null;

// ------------------------------------------------------------------ types --

export interface SessionUser {
  id: string;
  email: string;
  display_name: string;
  avatar_url: string;
  is_active: boolean;
  first_name: string;
  middle_name: string;
  last_name: string;
  username: string;
}

export interface LoginResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  refresh_expires_in: number;
  roles: string[];
  permissions: string[];
  user: SessionUser;
  // legacy field kept so the old code that reads `body.token` still works.
  token: string;
}

export interface MeResponse {
  id: string;
  email: string;
  display_name: string;
  avatar_url: string;
  first_name: string;
  middle_name: string;
  last_name: string;
  username: string;
  is_active: boolean;
  roles: string[];
  permissions: string[];
  expires_at: number;
  session_id: string;
}

export interface SessionView {
  id: string;
  user_id: string;
  device: string | null;
  user_agent: string | null;
  ip: string | null;
  created_at: string;
  last_seen_at: string;
  expires_at: string;
  revoked_at: string | null;
  current: boolean;
}

export interface ProfileAvatar {
  id: number;
  user_id: string;
  collection: string;
  filename: string;
  mime_type: string;
  size_bytes: number;
  created_at: string;
  updated_at: string;
}

export interface ProfileResponse {
  user: {
    id: string;
    email: string;
    display_name: string;
    avatar_url: string;
    first_name: string;
    middle_name: string;
    last_name: string;
    username: string;
  };
  profile: {
    user_id: string;
    display_name: string | null;
    phone: string | null;
    location: string | null;
    bio: string | null;
    avatar_file_id: number | null;
    created_at: string;
    updated_at: string;
  };
  avatars: ProfileAvatar[];
  current_avatar: ProfileAvatar | null;
}

export class ApiError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
  get notFound() {
    return this.status === 404;
  }
  get forbidden() {
    return this.status === 403;
  }
}

// ---------------------------------------------------------------- session -- (cookies + localStorage mirror)

function safeGet(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}
function safeRemove(key: string): void {
  try {
    localStorage.removeItem(key);
  } catch {
    /* private mode */
  }
}
function safeGetList(key: string): string[] {
  const raw = safeGet(key);
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.map(String) : [];
  } catch {
    return [];
  }
}
function safeSetList(key: string, value: string[]): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* private mode */
  }
}

export const session = {
  get token(): string | null {
    return safeGet(TOKEN_KEY);
  },
  get user(): SessionUser | null {
    const raw = safeGet(USER_KEY);
    if (!raw) return null;
    try {
      return JSON.parse(raw) as SessionUser;
    } catch {
      return null;
    }
  },
  get roles(): string[] {
    return safeGetList(ROLES_KEY);
  },
  get permissions(): string[] {
    return safeGetList(PERMS_KEY);
  },
  set(token: string, user: SessionUser): void {
    try {
      localStorage.setItem(TOKEN_KEY, token);
      localStorage.setItem(USER_KEY, JSON.stringify(user));
    } catch {
      /* private mode */
    }
  },
  setGrants(roles: string[], permissions: string[]): void {
    safeSetList(ROLES_KEY, roles);
    safeSetList(PERMS_KEY, permissions);
  },
  clear(): void {
    safeRemove(TOKEN_KEY);
    safeRemove(USER_KEY);
    safeRemove(ROLES_KEY);
    safeRemove(PERMS_KEY);
  },
};

let onUnauthorized: (() => void) | null = null;
export function handleUnauthorized(fn: () => void): void {
  onUnauthorized = fn;
}

// -------------------------------------------------------------- transport --

type Method = "GET" | "POST" | "PATCH" | "PUT" | "DELETE";

async function refreshSession(): Promise<boolean> {
  if (refreshInflight) return refreshInflight;
  refreshInflight = (async () => {
    try {
      const res = await fetch(`${API_BASE}/auth/refresh`, {
        method: "POST",
        credentials: "include",
      });
      if (!res.ok) {
        session.clear();
        onUnauthorized?.();
        return false;
      }
      const body = (await res.json()) as LoginResponse;
      session.set(body.access_token, body.user);
      session.setGrants(body.roles ?? [], body.permissions ?? []);
      return true;
    } catch {
      session.clear();
      onUnauthorized?.();
      return false;
    } finally {
      refreshInflight = null;
    }
  })();
  return refreshInflight;
}

async function request<T>(method: Method, path: string, body?: unknown): Promise<T> {
  const doFetch = async (): Promise<Response> => {
    const headers = new Headers({ Accept: "application/json" });
    if (body !== undefined) headers.set("Content-Type", "application/json");
    // Prefer the localStorage Bearer token for non-browser clients; cookies
    // travel automatically via credentials: include.
    const token = session.token;
    if (token) headers.set("Authorization", `Bearer ${token}`);
    return fetch(`${API_BASE}${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
      credentials: "include",
    });
  };

  let res: Response;
  try {
    res = await doFetch();
  } catch {
    throw new ApiError(0, "the gateway is unreachable");
  }

  // 401 on a non-auth path: try one refresh, then retry once.
  if (
    res.status === 401 &&
    !path.startsWith("/auth/") &&
    !path.startsWith("/auth/password-reset/")
  ) {
    const refreshed = await refreshSession();
    if (refreshed) {
      try {
        res = await doFetch();
      } catch {
        throw new ApiError(0, "the gateway is unreachable");
      }
    }
  }

  if (res.status === 204) return undefined as T;

  const text = await res.text();
  const payload = text ? parseJson(text) : {};

  if (!res.ok) {
    const message =
      typeof payload === "object" && payload !== null && "error" in payload
        ? String((payload as { error: unknown }).error)
        : `request failed (${res.status})`;
    throw new ApiError(res.status, message);
  }
  return payload as T;
}

function parseJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    throw new ApiError(502, "the gateway sent a malformed response");
  }
}

async function uploadRequest<T>(path: string, file: File, fields?: Record<string, string>): Promise<T> {
  const form = new FormData();
  form.append("file", file);
  if (fields) {
    for (const [key, val] of Object.entries(fields)) {
      form.append(key, val);
    }
  }
  const headers = new Headers();
  const token = session.token;
  if (token) headers.set("Authorization", `Bearer ${token}`);

  let res: Response;
  try {
    res = await fetch(`${API_BASE}${path}`, {
      method: "POST",
      headers,
      body: form,
      credentials: "include",
    });
  } catch {
    throw new ApiError(0, "the gateway is unreachable");
  }

  if (
    res.status === 401 &&
    !path.startsWith("/auth/") &&
    !path.startsWith("/auth/password-reset/")
  ) {
    const refreshed = await refreshSession();
    if (refreshed) {
      try {
        res = await fetch(`${API_BASE}${path}`, {
          method: "POST",
          headers,
          body: form,
          credentials: "include",
        });
      } catch {
        throw new ApiError(0, "the gateway is unreachable");
      }
    }
  }

  if (res.status === 204) return undefined as T;

  const text = await res.text();
  const payload = text ? parseJson(text) : {};

  if (!res.ok) {
    const message =
      typeof payload === "object" && payload !== null && "error" in payload
        ? String((payload as { error: unknown }).error)
        : `request failed (${res.status})`;
    throw new ApiError(res.status, message);
  }
  return payload as T;
}

export const api = {
  get: <T>(path: string) => request<T>("GET", path),
  post: <T>(path: string, body?: unknown) => request<T>("POST", path, body),
  patch: <T>(path: string, body?: unknown) => request<T>("PATCH", path, body),
  put: <T>(path: string, body?: unknown) => request<T>("PUT", path, body),
  del: <T>(path: string) => request<T>("DELETE", path),
  upload: <T>(path: string, file: File, fields?: Record<string, string>) =>
    uploadRequest<T>(path, file, fields),
};

// -------------------------------------------------------------- endpoints --

export interface Permission {
  id: number;
  module: string;
  action: string;
  description: string;
}

export interface SamplePassInfo {
  id: string;
  name: string;
  pass_style: string;
  description: string;
  colour: string;
}

export interface IssuedPassView {
  pass_type_id: string;
  serial_number: string;
  auth_token: string;
  download_url: string;
  qr_svg: string;
}

export interface WalletPassRow {
  pass_type_id: string;
  serial_number: string;
  updated_at: string;
}

// ----------------------------------------------------------------- storage --

export interface StoredFile {
  id: number;
  user_id: string;
  collection: string;
  filename: string;
  mime_type: string;
  size_bytes: number;
  created_at: string;
  updated_at: string;
}

// ------------------------------------------------------------------ system --

export interface SystemInfo {
  id: number;
  name: string;
  logo_url: string;
  version: string;
}

// ----------------------------------------------------------------- support --

export interface TicketView {
  id: string;
  category: "viewer_to_admin" | "admin_to_admin" | "viewer_to_viewer";
  sender_id: string;
  recipient_id: string | null;
  sender: string;
  recipient: string | null;
  subject: string;
  body: string;
  status: "open" | "closed";
  created_at: string;
  updated_at: string;
}

export interface Recipient {
  id: string;
  name: string;
}
