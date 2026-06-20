# YaakoDrive 運用・CLI設計書

## 方針
通常のユーザ操作はWeb UI/APIで行う。
一方で、サーバ管理者向けの保守処理はCLIクレートで実装する。
CLIはHTTPを経由せず、`app` / `infra` / `config` を利用して処理を実行する。

## CLIクレート
```text
backend/crates/cli
```

`server` と `cli` はどちらもエントリポイントである。

```text
server: HTTPサーバを起動する
cli: 管理コマンドを単発実行する
```

## 想定コマンド
```bash
yaakodrive-cli create-admin --username yaako
yaakodrive-cli cleanup-expired-tokens
yaakodrive-cli cleanup-trash --older-than-days 30
yaakodrive-cli check-storage-consistency
yaakodrive-cli cleanup-pending-uploads
```

## 初期管理者作成
最初の管理者ユーザはWeb UIから作れないため、CLIで作成する。

```bash
yaakodrive-cli create-admin --username yaako
```

パスワードは対話入力、または環境変数指定を検討する。
平文パスワードをコマンド履歴に残さないよう注意する。

## ユーザ作成
通常ユーザ作成は、MVPでは管理者Web画面/APIから行う。
自由登録は行わない。
必要であればCLIからも作成できるようにする。

## 期限切れRefresh Token削除
期限切れのRefresh Tokenを削除する。
定期実行してよい。

```bash
yaakodrive-cli cleanup-expired-tokens
```

## ゴミ箱物理削除
一定期間経過したゴミ箱内データを物理削除する。
MVPでは手動CLI実行でもよい。
将来的にはcron等で定期実行する。

```bash
yaakodrive-cli cleanup-trash --older-than-days 30
```

## ストレージ整合性チェック
DBと実ファイルの不整合を検出する。

検出対象:
- DBには存在するが実ファイルが存在しない
- 実ファイルは存在するがDBに対応する `file_contents` が存在しない
- `pending` 状態のまま一定時間以上経過したアップロード

```bash
yaakodrive-cli check-storage-consistency
```

MVPでは検出のみでもよい。
削除や修復を行う場合は、明示的なオプションを必要とする。

```bash
yaakodrive-cli check-storage-consistency --fix
```

## pendingアップロード掃除
アップロード失敗などで `pending` のまま残ったデータを掃除する。

```bash
yaakodrive-cli cleanup-pending-uploads
```

## バックアップ運用
バックアップはYaakoDrive本体とは別の運用として扱う。

- DBは `pg_dump` でバックアップ
- 実ファイルはrestic等でバックアップ
- バックアップ専用コンテナまたはホスト側cronで定期実行
- 年数回、復元試験を行う
- 復元後に `check-storage-consistency` を実行する

## ログ方針
Rust側は `tracing` / `tracing-subscriber` を利用する。
CLI実行時も、処理対象数、成功数、失敗数、エラー内容をログ出力する。

## 開発用コンテナ
開発用コンテナはリポジトリ全体を見渡せるようにする。
フロントエンド、バックエンド、config、sql、docsを同じ作業環境から参照できる構成にする。
