import { useState, useCallback } from 'react';
import * as api from '../api';

export function useAuth() {
  const [username, setUsername] = useState<string | null>(null);

  const login = useCallback(async (user: string, pass: string) => {
    const res = await api.login(user, pass);
    setUsername(res.username);
  }, []);

  const logout = useCallback(async () => {
    await api.logout();
    setUsername(null);
  }, []);

  return { username, login, logout };
}
