import { useState } from 'react';
import type { FormEvent } from 'react';
import * as api from '../api';
import type { NodeItem } from '../types';
import { ApiError } from '../api';

interface Props {
  onError: (msg: string) => void;
  onSuccess: (msg: string) => void;
}

export function SearchPanel({ onError, onSuccess }: Props) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<NodeItem[] | null>(null);
  const [loading, setLoading] = useState(false);

  const handleSearch = async (e: FormEvent) => {
    e.preventDefault();
    if (!query.trim()) return;
    setLoading(true);
    try {
      const items = await api.searchNodes(query.trim());
      setResults(items);
    } catch (e) {
      onError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleDownload = async (node: NodeItem) => {
    try {
      const { url } = await api.getDownloadUrl(node.id);
      const a = document.createElement('a');
      a.href = url;
      a.download = node.name;
      a.click();
      onSuccess(`「${node.name}」のダウンロードを開始しました`);
    } catch (e) {
      onError(e instanceof ApiError ? e.message : String(e));
    }
  };

  return (
    <div className="search-panel">
      <form className="search-form" onSubmit={handleSearch}>
        <input
          className="input search-input"
          type="search"
          placeholder="ファイル・フォルダ名を検索…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <button className="btn btn-primary" type="submit" disabled={loading}>
          {loading ? '検索中…' : '検索'}
        </button>
      </form>

      {results !== null && (
        <div className="search-results">
          <p className="result-count">
            {results.length === 0 ? '該当なし' : `${results.length}件 ヒット`}
          </p>
          {results.length > 0 && (
            <div className="node-list">
              <div className="node-list-header">
                <span className="col-name">名前</span>
                <span className="col-date">更新日時</span>
                <span className="col-actions">操作</span>
              </div>
              {results.map((node) => (
                <div key={node.id} className="node-row">
                  <span className="col-name node-name-cell">
                    <span className="node-icon">
                      {node.node_type === 'folder' ? '📁' : '📄'}
                    </span>
                    {node.name}
                  </span>
                  <span className="col-date node-date">
                    {new Date(node.updated_at).toLocaleString('ja-JP')}
                  </span>
                  <span className="col-actions">
                    <div className="node-actions">
                      <code className="node-id-badge">{node.id.slice(0, 8)}</code>
                      {node.node_type === 'file' && (
                        <button
                          className="action-btn"
                          onClick={() => handleDownload(node)}
                          title="ダウンロード"
                        >
                          ⬇
                        </button>
                      )}
                    </div>
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
