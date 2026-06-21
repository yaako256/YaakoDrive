# 開発日記

## 2026年06月20日
- ブレスト & 仮設計書完成
- 設計書の完成
- Docker周りの整備が完成。開発に移れるようになった。
- configクレート作成
- serverのエントリポイント作成
- apiのヘルスチェック部分作成
- 各SQLファイルの作成
- makefileを使った、ホストからバックエンド経由でのSQL操作の確認
- authクレート、nodeクレート、repositoryクレートの型定義を作成

warkspace周りの作成をした。
serverの起動や、別ターミナルでの`curl -s http://localhost:8080/api/health | jq`確認をした。
SQL関連を確認した。
型定義をした。
つまり、実装順序ののPhase1と2と3と4が完成した。