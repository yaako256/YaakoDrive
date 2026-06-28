import type { NodeItem } from '../types';

interface Props {
  node: NodeItem;
  onRename: (node: NodeItem) => void;
  onMove: (node: NodeItem) => void;
  onDelete: (node: NodeItem) => void;
  onDownload: (node: NodeItem) => void;
}

export function NodeActions({ node, onRename, onMove, onDelete, onDownload }: Props) {
  return (
    <div className="node-actions" onClick={(e) => e.stopPropagation()}>
      {node.node_type === 'file' && (
        <button className="action-btn" onClick={() => onDownload(node)} title="ダウンロード">
          ⬇
        </button>
      )}
      <button className="action-btn" onClick={() => onRename(node)} title="名前変更">
        ✏️
      </button>
      <button className="action-btn" onClick={() => onMove(node)} title="移動">
        ↪
      </button>
      <button className="action-btn action-btn-danger" onClick={() => onDelete(node)} title="ゴミ箱へ">
        🗑
      </button>
    </div>
  );
}
