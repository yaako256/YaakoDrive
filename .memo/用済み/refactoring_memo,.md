# リファクタリング点(候補なだけで、実行するかは要検討)

## NodeRepositoryの分割
NodeRepositoryをDashboardQueryServiceやQueryServiceに分ける(UseCase中心になってると思う。そのため、Query系（Dashboard・Search・Trash一覧など）をRepositoryから切り出す処理はこの後リファクタリングしようと思う)

## config化
config化した方がいいができていない部分が多い。
以下にconfig化した方がよさそうなものを挙げるが、これ以外にもありそうだ。
- 通常ユーザの制限容量(現在10GB)
- 管理者ユーザの制限容量(現在20GB)

## 論理削除の復元で独立削除されたファイルも復元される問題
`soft_delete_with_descendants` のSQL：
```sql
UPDATE nodes SET deleted_at = $2, updated_at = $2
WHERE id IN (SELECT id FROM descendants)
AND deleted_at IS NULL  -- ← すでに削除済みのものは触らない（正しい）
```

例えば、フォルダAの中にファイルCがあり、Cを先に個別削除した後、フォルダAを削除したとする。このとき Cの`deleted_at` は保持される。

しかし `restore_with_descendants` は：
```sql
UPDATE nodes SET deleted_at = NULL
WHERE id IN (SELECT id FROM descendants)  -- ← 全子孫を無条件で復元
```

**フォルダAを復元すると、先に個別削除したCも一緒に復元されてしまう**。


## エラー伝搬の仮定義
`api/src/error.rs`：
```rust
// 仮定義
AppError::Storage(msg) => (StatusCode::CONFLICT, "storage", msg),
AppError::Node(msg) => (StatusCode::CONFLICT, "node", msg),
```
`Storage` エラーは通常 `500 Internal Server Error`、`Node` エラーは内容によって `400/409/500` と分けるべきです。フロントエンド開発に入る前に整理が必要。

## ゴミ箱復元でのDB更新が2回必要な場面で設計が複雑
`restore_node.rs` は：
1. `restore_with_descendants()` で `deleted_at=NULL` をセット（全子孫）
2. `update()` で `name` を更新（リネームした場合のみ）

という2ステップになる。トランザクションで保護されていないため、1が成功して2が失敗した場合、名前変更なしで復元される状態になる。UnitOfWorkを使って原子的に行うか、`restore_with_descendants_and_rename` のような専用クエリを検討した方が良いか悩みどころである。

## `DownloadTokenStore` に期限切れトークンのクリーンアップ機構がない
現状は `consume` のときだけ期限チェックしますが、消費されなかった期限切れトークンがメモリに残り続ける。長期稼働サーバでは積み重なる。tokioのspawnを使った定期クリーンアップなどの方法で解決したい。

## 通常のユーザを作る機構がない
一旦CLIコマンドで作ってしまうか、APIを実装するかは悩みどころである。
友人が使う時、その友人のパスワードは僕は知らない方がいいとは思う。

## Cargo.tomlについて
リファクタリングを進めた結果、必要じゃなくなった依存関係が生まれている気がするので、それを削除したい。
また、tokioなどがが"full"になっていたりするので、そこを最適化してもいいかもしれない










# 実際にすでにリファクタリングした箇所
## validate_name() を node crateへ移動
validate_name()を`node/name.rs`に移動し、Nodeにrename()を追加した。
これにより、ドメインルールをNodeへ寄せれた。

## NodeRow型を作成してquery_asに
NodeRow型を作成してquery_asにした。これにより、node_repository.rsが少し整理された。

## Nodeのドメインロジックを育てる
Node型に、rename()やmove_to()を作り、ドメインロジックを育てた。それに伴い、UseCaseやhandlerがリファクタリングされた。
また、ドメイン情報をすべてprivateにし、ゲッタ関数を作成して、保守性を高めた

## User型のドメインロジックを育てた
User型にnew_userやnew_adminを作り、ユーザ名やパスワードのバリテーションも追加した。
create_adminのユースケースが簡単化された。

# User型やRefreshTokenのドメインロジックも育てた