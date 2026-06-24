# APIの確認用コマンドメモ

## 注意点
- サーバポートはenvで変更してる可能性があるので確認すること
- それがcompose.yamlと一致しているか確認すること

## コマンド群
```bash
# ヘルスチェック
curl -s http://localhost:9090/api/health | jq

# ログイン
curl -s -c cookies.txt -X POST http://localhost:9090/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"yaako-admin","password":"yourpassword"}' | jq

# レスポンス例
# { "data": { "username": "yaako-admin" }, "error": null }

# Refresh
curl -s -b cookies.txt -c cookies.txt -X POST \
  http://localhost:9090/api/auth/refresh | jq

# Logout
curl -s -b cookies.txt -X POST \
  http://localhost:9090/api/auth/logout | jq

```

