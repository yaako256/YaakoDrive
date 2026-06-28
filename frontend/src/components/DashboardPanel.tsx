import { useEffect, useState } from 'react';
import * as api from '../api';
import type { DashboardData } from '../types';

function fmtBytes(b: number): string {
  if (b >= 1024 ** 3) return `${(b / 1024 ** 3).toFixed(1)} GB`;
  if (b >= 1024 ** 2) return `${(b / 1024 ** 2).toFixed(1)} MB`;
  if (b >= 1024) return `${(b / 1024).toFixed(1)} KB`;
  return `${b} B`;
}

export function DashboardPanel() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [err, setErr] = useState('');

  useEffect(() => {
    api.getDashboard()
      .then(setData)
      .catch((e) => setErr(e.message));
  }, []);

  if (err) return <div className="panel-error">ダッシュボード取得エラー: {err}</div>;
  if (!data) return <div className="panel-loading">読み込み中…</div>;

  const pct = data.limit_bytes > 0
    ? Math.min(100, Math.round((data.used_bytes / data.limit_bytes) * 100))
    : 0;

  return (
    <div className="dashboard">
      <h2 className="section-title">ダッシュボード</h2>

      <div className="dash-grid">
        <div className="dash-card">
          <span className="dash-label">使用容量</span>
          <span className="dash-value">{fmtBytes(data.used_bytes)}</span>
          <span className="dash-sub">/ {fmtBytes(data.limit_bytes)}</span>
          <div className="progress-bar">
            <div className="progress-fill" style={{ width: `${pct}%` }} />
          </div>
          <span className="dash-pct">{pct}%</span>
        </div>
        <div className="dash-card">
          <span className="dash-label">ファイル数</span>
          <span className="dash-value">{data.file_count.toLocaleString()}</span>
        </div>
        <div className="dash-card">
          <span className="dash-label">フォルダ数</span>
          <span className="dash-value">{data.folder_count.toLocaleString()}</span>
        </div>
      </div>

      {data.mime_stats.length > 0 && (
        <div className="dash-mime">
          <h3 className="dash-mime-title">ファイル種別</h3>
          <table className="mime-table">
            <thead>
              <tr><th>MIME Type</th><th>件数</th></tr>
            </thead>
            <tbody>
              {data.mime_stats.map((m) => (
                <tr key={m.mime_type}>
                  <td><code>{m.mime_type}</code></td>
                  <td>{m.count}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
