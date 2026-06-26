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

## Node関連作成時のチェックコマンド群
```bash
# 2つ目のアカウントを作成
make create-admin USERNAME=yaako-admin2
> yaakoadmin2

# ユーザDB確認
make user

# ログインしてCookieを取得(1ユーザ目)
curl -s -c cookies_1.txt -X POST http://localhost:9090/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"yaako-admin","password":"yaakoadmin"}' | jq

# ログインしてCookieを取得(2ユーザ目)
curl -s -c cookies_2.txt -X POST http://localhost:9090/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"yaako-admin2","password":"yaakoadmin2"}' | jq


# ルート直下にフォルダ作成
curl -s -b cookies_1.txt -X POST http://localhost:9090/api/nodes/folders \
  -H "Content-Type: application/json" \
  -d '{"name":"documents"}' | jq

# ルートの子一覧取得
curl -s -b cookies_1.txt http://localhost:9090/api/nodes | jq

# フォルダ配下にフォルダ作成（{id}は上で取得したid）
curl -s -b cookies_1.txt -X POST http://localhost:9090/api/nodes/{id}/folders \
  -H "Content-Type: application/json" \
  -d '{"name":"work"}' | jq

# フォルダ配下の子一覧を取得
curl -s -b cookies_1.txt http://localhost:9090/api/nodes/{id}/children | jq

# リネーム
curl -s -b cookies_1.txt -X PATCH http://localhost:9090/api/nodes/{id}/rename \
  -H "Content-Type: application/json" \
  -d '{"name":"personal"}' | jq

# 移動（ルート直下へ）
curl -s -b cookies_1.txt -X PATCH http://localhost:9090/api/nodes/{id}/move \
  -H "Content-Type: application/json" \
  -d '{"new_parent_id":null}' | jq

# 自分自身への移動で400エラーが出ることを確認
curl -s -b cookies_1.txt -X PATCH http://localhost:9090/api/nodes/{id}/move \
  -H "Content-Type: application/json" \
  -d '{"new_parent_id":"{id}"}' | jq

# 別ユーザに移動してみたら404エラーが出ることを確認
curl -s -b cookies_2.txt -X PATCH http://localhost:9090/api/nodes/{id}/move \
  -H "Content-Type: application/json" \
  -d '{"new_parent_id":null}' | jq

# 削除（ゴミ箱へ）(deleted_at確認)
curl -s -b cookies.txt -X DELETE http://localhost:9090/api/nodes/{id} | jq

# ログアウト
curl -s -b cookies_1.txt -X POST \
  http://localhost:9090/api/auth/logout | jq

# 認証なしリクエストで401エラーが返る
curl -s -b cookies_1.txt -X PATCH http://localhost:9090/api/nodes/{id}/rename \
  -H "Content-Type: application/json" \
  -d '{"name":"personal2"}' | jq

# nodeテーブルの確認
make node
```



## アップロードテスト関連のapi検証フロー
```bash
# 実行場所にあらかじめupload_test.txtなどを用意しておく

# ヘルスチェック
curl -s http://localhost:9090/api/health | jq

# ログインしてCookieを取得(1ユーザ目)
curl -s -c cookies_1.txt -X POST http://localhost:9090/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"yaako-admin","password":"yaakoadmin"}' | jq

# refresh
curl -s -b cookies_1.txt -c cookies.txt -X POST \
  http://localhost:9090/api/auth/refresh | jq

# Logout
curl -s -b cookies_1.txt -X POST \
  http://localhost:9090/api/auth/logout | jq






# ルートの子一覧取得(確認)
# 現在、personalというフォルダがルートにあるはずである。
curl -s -b cookies_1.txt http://localhost:9090/api/nodes | jq

# personalフォルダ内の子一覧取得(確認)
# まだ何もないはずである
curl -s -b cookies_1.txt http://localhost:9090/api/nodes/{id}/children | jq

# ルート直下へアップロード
curl -s -b cookies_1.txt -X POST http://localhost:9090/api/nodes/upload \
  -F "file=@./upload_test_file/test.txt" | jq

# もう一回ルート直下へアップロード
# 同名ファイルで 409 AlreadyExists が返ることの確認
curl -s -b cookies_1.txt -X POST http://localhost:9090/api/nodes/upload \
  -F "file=@./upload_test_file/test.txt" | jq

# フォルダにアップロード
curl -s -b cookies_1.txt -X POST http://localhost:9090/api/nodes/{folder_id}/upload \
  -F "file=@./upload_test_file/test_1.txt" | jq

# フォルダ内に画像ファイルをアップロード
curl -s -b cookies_1.txt -X POST http://localhost:9090/api/nodes/{folder_id}/upload \
  -F "file=@./upload_test_file/test_photo.txt" | jq

# フォルダに大きめのファイルをアップロード
curl -s -b cookies_1.txt -X POST http://localhost:9090/api/nodes/{folder_id}/upload \
  -F "file=@./upload_test_file/test_large_photo.jpg" | jq

# ダウンロードURL取得
curl -s -b cookies.txt http://localhost:9090/api/nodes/{node_id}/download-url | jq

# 実ファイル取得(認証不要)
# これをブラウザのurl欄に入れてみる
# http://localhost:9090/api/files/download/{token}

# もう一度ダウンロードしてみる
# 404 Not Foundが出るはず
# http://localhost:9090/api/files/download/{token}


# まだやってないもの
# 10MBのファイルを送信(6MBはやった)
# アップロード失敗時に tmp/ に残骸が残らない ← 残ってないものはわからん
```

