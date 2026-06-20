# 開発日記

## 2026年06月20日
- ブレスト & 仮設計書完成
- 設計書の完成
- Docker周りの整備が完成。開発に移れるようになった。
- configクレート作成
- serverのエントリポイント作成
- apiのヘルスチェック部分作成

warkspace周りの作成をした。
serverの起動や、別ターミナルでの`curl -s http://localhost:8080/api/health | jq`確認をした。
つまり、実装順序ののPhase1と2が完成した。