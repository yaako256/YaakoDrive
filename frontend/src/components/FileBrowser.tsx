import { useEffect, useState, useRef } from 'react';
import type { ChangeEvent } from 'react';
import * as api from '../api';
import type { NodeItem, Breadcrumb } from '../types';
import { NodeActions } from './NodeActions';
import { Modal } from './Modal';
import { ApiError } from '../api';

interface Props {
  onError: (msg: string) => void;
  onSuccess: (msg: string) => void;
}

export function FileBrowser({ onError, onSuccess }: Props) {
  // ─── ナビゲーション状態 ──────────────────────────────────────────────────
  const [breadcrumbs, setBreadcrumbs] = useState<Breadcrumb[]>([
    { id: null, name: 'マイドライブ' },
  ]);
  const [nodes, setNodes] = useState<NodeItem[]>([]);
  const [loading, setLoading] = useState(false);

  // ─── モーダル状態 ────────────────────────────────────────────────────────
  const [renameTarget, setRenameTarget] = useState<NodeItem | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [newFolderOpen, setNewFolderOpen] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [moveTarget, setMoveTarget] = useState<NodeItem | null>(null);
  const [moveParentId, setMoveParentId] = useState<string>('__root__');

  // ─── アップロード ────────────────────────────────────────────────────────
  const [uploading, setUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const currentId = breadcrumbs[breadcrumbs.length - 1].id;

  // ─── ノード一覧の取得 ────────────────────────────────────────────────────
  const reload = async () => {
    setLoading(true);
    try {
      const items = currentId == null
        ? await api.listRoot()
        : await api.listChildren(currentId);
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

  // ─── フォルダを開く ──────────────────────────────────────────────────────
  const openFolder = (node: NodeItem) => {
    setBreadcrumbs((prev) => [...prev, { id: node.id, name: node.name }]);
  };

  const navigateTo = (index: number) => {
    setBreadcrumbs((prev) => prev.slice(0, index + 1));
  };

  // ─── フォルダ作成 ────────────────────────────────────────────────────────
  const handleCreateFolder = async () => {
    if (!newFolderName.trim()) return;
    try {
      if (currentId == null) {
        await api.createRootFolder(newFolderName.trim());
      } else {
        await api.createFolder(currentId, newFolderName.trim());
      }
      onSuccess(`フォルダ「${newFolderName}」を作成しました`);
      setNewFolderOpen(false);
      setNewFolderName('');
      reload();
    } catch (e) {
      onError(e instanceof ApiError ? e.message : String(e));
    }
  };

  // ─── ファイルアップロード ────────────────────────────────────────────────
  const handleFileSelect = async (e: ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;
    setUploading(true);
    let failed = 0;
    for (const file of files) {
      try {
        await api.uploadFile(currentId, file);
      } catch {
        failed++;
      }
    }
    setUploading(false);
    if (fileInputRef.current) fileInputRef.current.value = '';
    if (failed === 0) {
      onSuccess(`${files.length}件アップロードしました`);
    } else {
      onError(`${failed}件のアップロードに失敗しました`);
    }
    reload();
  };

  // ─── 名前変更 ────────────────────────────────────────────────────────────
  const openRename = (node: NodeItem) => {
    setRenameTarget(node);
    setRenameValue(node.name);
  };

  const handleRename = async () => {
    if (!renameTarget || !renameValue.trim()) return;
    try {
      await api.renameNode(renameTarget.id, renameValue.trim());
      onSuccess('名前を変更しました');
      setRenameTarget(null);
      reload();
    } catch (e) {
      onError(e instanceof ApiError ? e.message : String(e));
    }
  };

  // ─── 移動 ────────────────────────────────────────────────────────────────
  const openMove = (node: NodeItem) => {
    setMoveTarget(node);
    setMoveParentId('__root__');
  };

  const handleMove = async () => {
    if (!moveTarget) return;
    const newParentId = moveParentId === '__root__' ? null : moveParentId;
    try {
      await api.moveNode(moveTarget.id, newParentId);
      onSuccess('移動しました');
      setMoveTarget(null);
      reload();
    } catch (e) {
      onError(e instanceof ApiError ? e.message : String(e));
    }
  };

  // ─── ゴミ箱へ ────────────────────────────────────────────────────────────
  const handleDelete = async (node: NodeItem) => {
    if (!confirm(`「${node.name}」をゴミ箱に移動しますか？`)) return;
    try {
      await api.deleteNode(node.id);
      onSuccess(`「${node.name}」をゴミ箱へ移動しました`);
      reload();
    } catch (e) {
      onError(e instanceof ApiError ? e.message : String(e));
    }
  };

  // ─── ダウンロード ────────────────────────────────────────────────────────
  const handleDownload = async (node: NodeItem) => {
    try {
      const { url } = await api.getDownloadUrl(node.id);

      // ダウンロードURLをFull Urlにする
      window.location.href = url;

    } catch (e) {
      onError(e instanceof ApiError ? e.message : String(e));
    }
  };

  // ─── レンダリング ────────────────────────────────────────────────────────
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

      {/* ツールバー */}
      <div className="toolbar">
        <button className="btn btn-sm" onClick={() => setNewFolderOpen(true)}>
          📁 新しいフォルダ
        </button>
        <button
          className="btn btn-sm btn-primary"
          onClick={() => fileInputRef.current?.click()}
          disabled={uploading}
        >
          {uploading ? 'アップロード中…' : '⬆ ファイルを追加'}
        </button>
        <input
          ref={fileInputRef}
          type="file"
          multiple
          style={{ display: 'none' }}
          onChange={handleFileSelect}
        />
        <button className="btn btn-sm btn-ghost" onClick={reload}>
          🔄 再読み込み
        </button>
      </div>

      {/* ノード一覧 */}
      {loading ? (
        <div className="loading-msg">読み込み中…</div>
      ) : nodes.length === 0 ? (
        <div className="empty-msg">
          <span className="empty-icon">📂</span>
          <p>このフォルダは空です</p>
        </div>
      ) : (
        <div className="node-list">
          <div className="node-list-header">
            <span className="col-name">名前</span>
            <span className="col-date">更新日時</span>
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
                <NodeActions
                  node={node}
                  onRename={openRename}
                  onMove={openMove}
                  onDelete={handleDelete}
                  onDownload={handleDownload}
                />
              </span>
            </div>
          ))}
        </div>
      )}

      {/* フォルダ作成モーダル */}
      {newFolderOpen && (
        <Modal title="新しいフォルダ" onClose={() => setNewFolderOpen(false)}>
          <label className="field-label">
            フォルダ名
            <input
              className="input"
              type="text"
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleCreateFolder()}
              autoFocus
            />
          </label>
          <div className="modal-actions">
            <button className="btn btn-primary" onClick={handleCreateFolder}>作成</button>
            <button className="btn btn-ghost" onClick={() => setNewFolderOpen(false)}>キャンセル</button>
          </div>
        </Modal>
      )}

      {/* 名前変更モーダル */}
      {renameTarget && (
        <Modal title="名前変更" onClose={() => setRenameTarget(null)}>
          <label className="field-label">
            新しい名前
            <input
              className="input"
              type="text"
              value={renameValue}
              onChange={(e) => setRenameValue(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleRename()}
              autoFocus
            />
          </label>
          <div className="modal-actions">
            <button className="btn btn-primary" onClick={handleRename}>変更</button>
            <button className="btn btn-ghost" onClick={() => setRenameTarget(null)}>キャンセル</button>
          </div>
        </Modal>
      )}

      {/* 移動モーダル */}
      {moveTarget && (
        <Modal title={`「${moveTarget.name}」を移動`} onClose={() => setMoveTarget(null)}>
          <label className="field-label">
            移動先フォルダID
            <p className="field-hint">
              移動先フォルダの ID を入力するか、ルートに移動する場合は空欄にしてください。
            </p>
            <input
              className="input input-mono"
              type="text"
              placeholder="フォルダID（空欄でルート）"
              value={moveParentId === '__root__' ? '' : moveParentId}
              onChange={(e) => setMoveParentId(e.target.value.trim() || '__root__')}
            />
          </label>
          <div className="modal-actions">
            <button className="btn btn-primary" onClick={handleMove}>移動</button>
            <button className="btn btn-ghost" onClick={() => setMoveTarget(null)}>キャンセル</button>
          </div>
        </Modal>
      )}
    </div>
  );
}
