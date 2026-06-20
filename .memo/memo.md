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