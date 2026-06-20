# YaakoDrive API設計書

## 方針
APIはJSONベースとし、ファイルアップロードはmultipartを利用する。
レスポンスは成功・失敗ともにエンベロープ形式で返す。

## レスポンス形式
```json
{ "data": { }, "error": null }
```

```json
{ "data": null, "error": { "code": "not_found", "message": "指定されたノードが存在しません" } }
```

## エラーコード
| code | HTTPステータス | 意味 |
|------|---------------|------|
| unauthorized | 401 | 未認証 |
| forbidden | 403 | 権限不足 |
| not_found | 404 | リソースが存在しない |
| already_exists | 409 | 同名ファイル・フォルダの重複 |
| storage_limit_exceeded | 409 | ストレージ上限超過 |
| invalid_request | 422 | リクエスト内容が不正 |
| internal_error | 500 | サーバ内部エラー |

## 認証API
| メソッド | パス | 説明 |
|---------|------|------|
| POST | `/api/auth/login` | ログイン。Access Token・Refresh TokenをCookieにセット |
| POST | `/api/auth/refresh` | Token再発行。Refresh Tokenをローテーション |
| POST | `/api/auth/logout` | ログアウト。Cookie削除とRefresh Token revoke |

## ノードAPI
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/nodes` | ルート直下の一覧取得 |
| GET | `/api/nodes/{id}/children` | 指定フォルダ直下の一覧取得 |
| GET | `/api/nodes/{id}` | ノード情報取得 |
| PATCH | `/api/nodes/{id}/rename` | リネーム |
| PATCH | `/api/nodes/{id}/move` | 移動 |
| DELETE | `/api/nodes/{id}` | ゴミ箱へ移動 |

## ファイルAPI
| メソッド | パス | 説明 |
|---------|------|------|
| POST | `/api/nodes/{id}/upload` | 指定フォルダへファイルアップロード |
| GET | `/api/nodes/{id}/download-url` | 一時ダウンロードURL発行 |
| GET | `/api/files/download/{token}` | 実ファイル取得 |

一時ダウンロードURLはMVPではサーバメモリ上で管理する。
MVPは単一インスタンス前提のため、サーバ再起動で未使用URLは失効する。

## フォルダAPI
| メソッド | パス | 説明 |
|---------|------|------|
| POST | `/api/nodes/{id}/folders` | 指定フォルダ配下にフォルダ作成 |

## ゴミ箱API
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/trash` | ゴミ箱一覧取得 |
| POST | `/api/trash/{id}/restore` | ゴミ箱から復元 |
| DELETE | `/api/trash/{id}` | 物理削除 |

## 検索API
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/search?q={keyword}` | ファイル・フォルダ名検索 |

## Dashboard API
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/dashboard` | 使用容量・ファイル数などの統計情報取得 |

## 管理者API
| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/admin/users` | ユーザ一覧取得 |
| POST | `/api/admin/users` | ユーザ作成 |
| PATCH | `/api/admin/users/{id}` | ユーザ情報変更 |

## Cookie設定
| 属性 | 本番 | 開発 | 理由 |
|------|------|------|------|
| HttpOnly | true | true | JSから読めなくする |
| Secure | true | false可 | HTTPSのみ送信。本番では必須 |
| SameSite | Strict | Strict | CSRF対策 |
| Path(Access Token) | `/api` | `/api` | API全体で利用 |
| Path(Refresh Token) | `/api/auth/refresh` | `/api/auth/refresh` | Refresh Token送信範囲を限定 |

## アップロードAPI方針
MVPでもストリーミング前提とする。
API層ではmultipartを受け取り、ファイル全体をメモリに載せずにStorageServiceへ渡す。
最大ファイルサイズはconfigで指定する。
