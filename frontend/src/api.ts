import type {
  ApiResponse,
  NodeItem,
  LoginResponse,
  DashboardData,
  DownloadUrlData,
} from './types';

// ─── fetch ラッパー ──────────────────────────────────────────────────────────
// リフレッシュ中かどうかのフラグ（二重実行防止）
let isRefreshing = false;

async function apiFetch<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const res = await fetch(path, {
    credentials: 'include', // Cookie を常に送る
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
    ...options,
  });

  // 401 かつリフレッシュエンドポイント自体でなければリフレッシュを試みる
  if (res.status === 401 && !path.includes('/api/auth/') && !isRefreshing) {
    isRefreshing = true;
    try {
      const refreshRes = await fetch('/api/auth/refresh', {
        method: 'POST',
        credentials: 'include',
      });
      if (!refreshRes.ok) {
        // リフレッシュも失敗 → ログイン画面へ飛ばす
        window.location.href = '/';
        throw new ApiError('unauthorized', 'セッションが切れました', 401);
      }
    } finally {
      isRefreshing = false;
    }

    // リフレッシュ成功 → 元のリクエストを1回だけ再試行
    const retryRes = await fetch(path, {
      credentials: 'include',
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      ...options,
    });
    const retryJson: ApiResponse<T> = await retryRes.json();
    if (retryJson.error) {
      throw new ApiError(retryJson.error.code, retryJson.error.message, retryRes.status);
    }
    return retryJson.data as T;
  }


  const json: ApiResponse<T> = await res.json();

  if (json.error) {
    throw new ApiError(json.error.code, json.error.message, res.status);
  }

  return json.data as T;
}

export class ApiError extends Error {
  code: string;
  status: number;
  constructor(code: string, message: string, status: number) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
    this.status = status;
  }
}

// ─── 認証 ───────────────────────────────────────────────────────────────────

export const login = (username: string, password: string) =>
  apiFetch<LoginResponse>('/api/auth/login', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  });

export const logout = () =>
  apiFetch<null>('/api/auth/logout', { method: 'POST' });

export const refresh = () =>
  apiFetch<null>('/api/auth/refresh', { method: 'POST' });

// ─── ノード ─────────────────────────────────────────────────────────────────

export const listRoot = () =>
  apiFetch<NodeItem[]>('/api/nodes');

export const listChildren = (id: string) =>
  apiFetch<NodeItem[]>(`/api/nodes/${id}/children`);

export const getNode = (id: string) =>
  apiFetch<NodeItem>(`/api/nodes/${id}`);

export const createRootFolder = (name: string) =>
  apiFetch<NodeItem>('/api/nodes/folders', {
    method: 'POST',
    body: JSON.stringify({ name }),
  });

export const createFolder = (parentId: string, name: string) =>
  apiFetch<NodeItem>(`/api/nodes/${parentId}/folders`, {
    method: 'POST',
    body: JSON.stringify({ name }),
  });

export const renameNode = (id: string, name: string) =>
  apiFetch<NodeItem>(`/api/nodes/${id}/rename`, {
    method: 'PATCH',
    body: JSON.stringify({ name }),
  });

export const moveNode = (id: string, newParentId: string | null) =>
  apiFetch<NodeItem>(`/api/nodes/${id}/move`, {
    method: 'PATCH',
    body: JSON.stringify({ new_parent_id: newParentId }),
  });

export const deleteNode = (id: string) =>
  apiFetch<null>(`/api/nodes/${id}`, { method: 'DELETE' });

// ─── ファイル ────────────────────────────────────────────────────────────────

export const uploadFile = (parentId: string | null, file: File) => {
  const form = new FormData();
  form.append('file', file, file.name);
  const url = parentId == null
    ? '/api/nodes/upload'
    : `/api/nodes/${parentId}/upload`;
  return apiFetch<NodeItem>(url, {
    method: 'POST',
    headers: {}, // Content-Type は FormData に任せる (ヘッダー上書きしない)
    body: form,
  });
};

export const getDownloadUrl = (id: string) =>
  apiFetch<DownloadUrlData>(`/api/nodes/${id}/download-url`);

// ─── ゴミ箱 ─────────────────────────────────────────────────────────────────

export const listTrash = () =>
  apiFetch<NodeItem[]>('/api/trash');

export const listTrashChildren = (id: string) =>
  apiFetch<NodeItem[]>(`/api/trash/${id}/children`);

export const restoreNode = (id: string, newName?: string) =>
  apiFetch<NodeItem>(`/api/trash/${id}/restore`, {
    method: 'POST',
    body: JSON.stringify({ new_name: newName ?? null }),
  });

export const hardDeleteNode = (id: string) =>
  apiFetch<null>(`/api/trash/${id}`, { method: 'DELETE' });

// ─── 検索 ───────────────────────────────────────────────────────────────────

export const searchNodes = (q: string) =>
  apiFetch<NodeItem[]>(`/api/search?q=${encodeURIComponent(q)}`);

// ─── ダッシュボード ──────────────────────────────────────────────────────────

export const getDashboard = () =>
  apiFetch<DashboardData>('/api/dashboard');

// ─── ヘルスチェック ──────────────────────────────────────────────────────────

export const healthCheck = () =>
  fetch('/api/health').then((r) => r.json() as Promise<{ status: string }>);
