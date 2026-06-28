import { useState } from 'react';
import { LoginPage } from './components/LoginPage';
import { FileBrowser } from './components/FileBrowser';
import { TrashPanel } from './components/TrashPanel';
import { SearchPanel } from './components/SearchPanel';
import { DashboardPanel } from './components/DashboardPanel';
import { ToastContainer } from './components/ToastContainer';
import { useAuth } from './hooks/useAuth';
import { useToast } from './hooks/useToast';
import { ApiError } from './api';

type Tab = 'files' | 'trash' | 'search' | 'dashboard';

export default function App() {
  const { username, login, logout } = useAuth();
  const { toasts, push, dismiss } = useToast();
  const [tab, setTab] = useState<Tab>('files');

  const onError = (msg: string) => push('error', msg);
  const onSuccess = (msg: string) => push('success', msg);

  const handleLogout = async () => {
    try {
      await logout();
      push('info', 'ログアウトしました');
    } catch (e) {
      onError(e instanceof ApiError ? e.message : String(e));
    }
  };

  if (!username) {
    return (
      <>
        <LoginPage onLogin={login} />
        <ToastContainer toasts={toasts} onDismiss={dismiss} />
      </>
    );
  }

  const navItems: { id: Tab; label: string; icon: string }[] = [
    { id: 'files',     label: 'マイドライブ', icon: '🗂' },
    { id: 'search',    label: '検索',          icon: '🔍' },
    { id: 'trash',     label: 'ゴミ箱',        icon: '🗑' },
    { id: 'dashboard', label: 'ダッシュボード', icon: '📊' },
  ];

  return (
    <div className="app-layout">
      {/* サイドバー */}
      <aside className="sidebar">
        <div className="sidebar-logo">
          <span className="logo-mark">YD</span>
          <span className="logo-text">YaakoDrive</span>
          <span className="logo-badge">dev</span>
        </div>
        <nav className="sidebar-nav">
          {navItems.map((item) => (
            <button
              key={item.id}
              className={`nav-item ${tab === item.id ? 'nav-item-active' : ''}`}
              onClick={() => setTab(item.id)}
            >
              <span className="nav-icon">{item.icon}</span>
              {item.label}
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          <div className="user-info">
            <span className="user-avatar">{username[0].toUpperCase()}</span>
            <span className="user-name">{username}</span>
          </div>
          <button className="btn btn-sm btn-ghost logout-btn" onClick={handleLogout}>
            ログアウト
          </button>
        </div>
      </aside>

      {/* メインエリア */}
      <main className="main-area">
        <div className="main-content">
          {tab === 'files'     && <FileBrowser   onError={onError} onSuccess={onSuccess} />}
          {tab === 'trash'     && <TrashPanel    onError={onError} onSuccess={onSuccess} />}
          {tab === 'search'    && <SearchPanel   onError={onError} onSuccess={onSuccess} />}
          {tab === 'dashboard' && <DashboardPanel />}
        </div>
      </main>

      <ToastContainer toasts={toasts} onDismiss={dismiss} />
    </div>
  );
}
