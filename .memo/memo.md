# toDoメモ







# メモ
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