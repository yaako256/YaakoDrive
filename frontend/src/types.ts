// ─── API レスポンス共通型 ────────────────────────────────────────────────────

export interface ApiResponse<T> {
  data: T | null;
  error: { code: string; message: string } | null;
}

// ─── ノード ─────────────────────────────────────────────────────────────────

export type NodeType = 'file' | 'folder';

export interface NodeItem {
  id: string;
  parent_id: string | null;
  name: string;
  node_type: NodeType;
  created_at: string;
  updated_at: string;
}

// ─── 認証 ───────────────────────────────────────────────────────────────────

export interface LoginResponse {
  username: string;
}

// ─── ダッシュボード ──────────────────────────────────────────────────────────

export interface MimeStat {
  mime_type: string;
  count: number;
}

export interface DashboardData {
  used_bytes: number;
  limit_bytes: number;
  file_count: number;
  folder_count: number;
  mime_stats: MimeStat[];
}

// ─── ダウンロード URL ────────────────────────────────────────────────────────

export interface DownloadUrlData {
  url: string;
}

// ─── パンくずリスト ──────────────────────────────────────────────────────────

export interface Breadcrumb {
  id: string | null; // null = root
  name: string;
}
