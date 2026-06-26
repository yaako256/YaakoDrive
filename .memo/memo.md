# toDoメモ
- PCを再起動したらコンテナが立ち上がらなくなっている。restartするようにする。
- 存在しないファイル名をアップロードしようとした場合、エラーレスポンスすら来ないのは仕様か。
- ダウンロードはストリーミングになっているが、アップロードはストリーミングされていない。メモリ使用量がすごいことになる可能性がある。そのため、アップロードもストリーミングで頑張るようにする

# 長い目で見たtoDoメモ(リファクタリング対象)
- 全体がUTC管理になっている。jstにした方がいいのではないか？
- 現在だと、全体で同じフォルダ名等が使えないかもしれない。同一フォルダ内だけでUNIQE処理をしたい。
- Nodeの移動で、Nodeの`updated_at`は更新するべきか。中身は更新されていないので、`updata_at`は何か違うのではないか。
- Nodeで、ルートフォルダをNoneではなく、固有のIDにすることで、apiが一括化されてより良くなるのではないか。
- appがsqlxに依存している問題。infraに専用のアップロードRepositoryを作るのが理想


# 備忘録メモ
## `migrations/`の置き場所について
設計指南書や設計書から、このように変更した。
```
元:
`backend/migrations/`
↓
現在:
`sql/migrations/`
```
理由
- `migrations/`はRustの責務ではなく、DB管理の責務である点
- 将来的に補助SQLや初期データ投入スクリプトも同じ場所における点
- Rustのworkspaceのクレートと混在せず、見通しが良い点


## `APP__JWT__SECRET`の生成方法について
例えばこの方法で、ランダムな強い文字列が生成できる。
```
openssl rand -base64 48
```

## 本番用データの保存場所について
以下の場所にバインドマウントしている
```yaml
# sql保存先
- /srv/yaakodrive/postgres:/var/lib/postgresql/data

# 実ファイル保存先
- /srv/yaakodrive/data:/data
```
これはホストの絶対パスである。  
`/srv/`はLinuxのFHS(ファイルシステム階層標準)で「サービスが提供するデータを置く場所」として定義されているディレクトリである。  
一応ホストの普段手を出さないところが汚れていることを記憶しておく。  
また、開発用データは名前付きVolumeに保存しているため、外からは見えない。


## Dockerの環境変数について
Docker Composeの**マージ順序**が原因でした。
複数のcomposeファイルをマージするとき、環境変数の優先順位は以下のようになります。
```
1. environment: に直接書いた値  ← 最優先
2. env_file: から読んだ値
```
`compose.yaml` の `environment:` に `${POSTGRES_PASSWORD:-change-me}` と書いていたため、ホスト側のシェルで `POSTGRES_PASSWORD` が未定義のとき `change-me` に展開されました。その値が `environment:` として確定してしまい、`compose.dev.yaml` の `env_file:` から読んだ値より優先されてしまいました。
```
compose.yaml の environment: POSTGRES_PASSWORD=change-me  ← これが勝ってしまった
compose.dev.yaml の env_file: POSTGRES_PASSWORD=postgres_dev_password  ← 負けた
```

## sqlx::query!のエラーについて
`sqlx::query!`にエラーが出ていて、
```bash
cargo sqlx prepare --workspace
```
これをbackend内で実行してもエラーが消えない場合、rust-analyzerが原因の可能性がある。
`Ctrl+Shift+P → rust-analyzer: Restart Server`で治ると思われる

## CookieのRefreshToken管理場所について
RefreshTokenを独立させるため、`/api/auth/refresh`にCookieを登録していたが、それではlogoutで使えなかった。
そのため、RefreshTokenの管理場所を`/api/auth`にした。
その結果、login処理でも`RefreshToken`が動くことになった。
現在は`login`、`refresh`、`logout`の3つだからいいが、増えてきたら`/api/auth/refresh`のように、RefreshTokenの管理場所を限定した方がいいかもしれない。

## ファイルのアップロード/ダウンロードの設計について
設計上の観点などから以下のような変更をした。
```
変更前: TransactionContext が &dyn NodeRepository を返す
変更後: TransactionContext 自体がトランザクション操作のメソッドを持つ
```


# 後で設定化するけどまだ定数のもの
- アップロード上限10MB
- パスワード最低文字数8文字
- Nodeの最長名前長255文字
- /// ダウンロードトークンの有効期限 const TOKEN_TTL_SECS: u64 = 60;
- apiでの、ストレージエラーレスポンス

# キーワードメモメモ

ユートピア(ユトイピア？)ってやつを使うとapi関連の整合性を簡単に取れる？



find_ancestor_ids_impl の祖先の順序が不定です。現状のCTEは取得順が保証されないため、パンくずリストなど順序が重要な用途で使う場合は要注意です。もし順序が必要になったときは呼び出し側でソートするか、CTEに深さカラムを追加する方法があります。今すぐ直す必要はありませんが、頭の片隅に置いておくと良いです。
全体としては設計の意図が明確に実装に反映されていて、このまま進めて問題ないと思います。