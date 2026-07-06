import { useState, useCallback, useEffect } from 'react';
import * as api from '../api';

export function useAuth() {
  const [username, setUsername] = useState<string | null>(null);
  // 初期チェック中かどうか（チェック完了前にログイン画面を一瞬見せないため）
  const [checking, setChecking] = useState(true);

  // ページ読み込み時にセッションが有効かチェック
  useEffect(() => {
    api.getMe()
      .then((res) => setUsername(res.username))
      .catch(() => {
        // Cookie がない or 期限切れ → ログイン画面を表示
      })
      .finally(() => setChecking(false));
  }, []);

  const login = useCallback(async (user: string, pass: string) => {
    const res = await api.login(user, pass);
    setUsername(res.username);
  }, []);

  const logout = useCallback(async () => {
    await api.logout();
    setUsername(null);
  }, []);

  return { username, login, logout, checking };
}
