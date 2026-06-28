import { useEffect, useState } from 'react';
import * as api from '../api';
import type { NodeItem, Breadcrumb } from '../types';
import { Modal } from './Modal';
import { ApiError } from '../api';

interface Props {
  onError: (msg: string) => void;
  onSuccess: (msg: string) => void;
}

export function TrashPanel({ onError, onSuccess }: Props) {
  const [breadcrumbs, setBreadcrumbs] = useState<Breadcrumb[]>([
    { id: null, name: 'ゴミ箱' },
  ]);
  const [nodes, setNodes] = useState<NodeItem[]>([]);
  const [loading, setLoading] = useState(false);

  // 復元モーダル
  const [restoreTarget, setRestoreTarget] = useState<NodeItem | null>(null);
  const [restoreName, setRestoreName] = useState('');

  const currentId = breadcrumbs[breadcrumbs.length - 1].id;

  const reload = async () => {
    setLoading(true);
    try {
      const items = currentId == null
        ? await api.listTrash()
        : await api.listTrashChildren(currentId);
      setNodes(items);
    } catch (e) {
      onError(e instanceof ApiError ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    reload();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentId]);

  const openFolder = (node: NodeItem) => {
    setBreadcrumbs((prev) => [...prev, { id: node.id, name: node.name }]);
  };

  const navigateTo = (index: number) => {
    setBreadcrumbs((prev) => prev.slice(0, index + 1));
  };

  const openRestore = (node: NodeItem) => {
    setRestoreTarget(node);
    setRestoreName('');
  };

  const handleRestore = async () => {
    if (!restoreTarget) return;
    try {
      await api.restoreNode(
        restoreTarget.id,
        restoreName.trim() || undefined
      );
      onSuccess(`「${restoreTarget.name}」を復元しました`);
      setRestoreTarget(null);
      reload();
    } catch (e) {
      onError(e instanceof ApiError ? e.message : String(e));
    }
  };

  const handleHardDelete = async (node: NodeItem) => {
    if (!confirm(`「${node.name}」を完全削除します。元に戻せません。`)) return;
    try {
      await api.hardDeleteNode(node.id);
      onSuccess(`「${node.name}」を完全削除しました`);
      reload();
    } catch (e) {
      onError(e instanceof ApiError ? e.message : String(e));
    }
  };

  return (
    <div className="file-browser">
      {/* パンくず */}
      <nav className="breadcrumb">
        {breadcrumbs.map((b, i) => (
          <span key={i} className="breadcrumb-item">
            {i < breadcrumbs.length - 1 ? (
              <button className="breadcrumb-btn" onClick={() => navigateTo(i)}>
                {b.name}
              </button>
            ) : (
              <span className="breadcrumb-current">{b.name}</span>
            )}
            {i < breadcrumbs.length - 1 && <span className="breadcrumb-sep">/</span>}
          </span>
        ))}
      </nav>

      <div className="toolbar">
        <button className="btn btn-sm btn-ghost" onClick={reload}>🔄 再読み込み</button>
      </div>

      {loading ? (
        <div className="loading-msg">読み込み中…</div>
      ) : nodes.length === 0 ? (
        <div className="empty-msg">
          <span className="empty-icon">🗑</span>
          <p>ゴミ箱は空です</p>
        </div>
      ) : (
        <div className="node-list">
          <div className="node-list-header">
            <span className="col-name">名前</span>
            <span className="col-date">削除日時</span>
            <span className="col-actions">操作</span>
          </div>
          {nodes.map((node) => (
            <div key={node.id} className="node-row">
              <span
                className="col-name node-name-cell"
                onClick={() => node.node_type === 'folder' && openFolder(node)}
                style={{ cursor: node.node_type === 'folder' ? 'pointer' : 'default' }}
              >
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
                  <button className="action-btn" onClick={() => openRestore(node)} title="復元">
                    ↩
                  </button>
                  <button
                    className="action-btn action-btn-danger"
                    onClick={() => handleHardDelete(node)}
                    title="完全削除"
                  >
                    💣
                  </button>
                </div>
              </span>
            </div>
          ))}
        </div>
      )}

      {/* 復元モーダル */}
      {restoreTarget && (
        <Modal title={`「${restoreTarget.name}」を復元`} onClose={() => setRestoreTarget(null)}>
          <p className="field-hint">
            同名ファイルが存在する場合は別名を入力してください。空欄の場合は元の名前で復元します。
          </p>
          <label className="field-label">
            復元後の名前（省略可）
            <input
              className="input"
              type="text"
              placeholder={restoreTarget.name}
              value={restoreName}
              onChange={(e) => setRestoreName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleRestore()}
              autoFocus
            />
          </label>
          <div className="modal-actions">
            <button className="btn btn-primary" onClick={handleRestore}>復元</button>
            <button className="btn btn-ghost" onClick={() => setRestoreTarget(null)}>キャンセル</button>
          </div>
        </Modal>
      )}
    </div>
  );
}
