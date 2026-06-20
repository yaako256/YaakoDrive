# セットアップ備忘録

# バックエンド側(Rust)
## Workspaceを作成する
1. ルートディレクトリを作成する  
```bash
mkdir backend
```
2. クレート用ディレクトリを作成する  
```bash
mkdir crates
```
3. ルートにCargo.tomlを作成する  
以下を作成する
```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.dependencies]
```

## 子クレートを作成する
`--vcs none`を付けると、`.git/`および`gitignore`が生成されなくなる。
```bash
# クレート用ディレクトリに移動
cd crates

# バイナリクレート
cargo new server --bin --vcs none
cargo new cli --bin --vcs none

# ライブラリクレート
cargo new app --lib --vcs none
cargo new api --lib --vcs none
cargo new auth --lib --vcs none
cargo new identity --lib --vcs none
cargo new node --lib --vcs none
cargo new storage --lib --vcs none
cargo new repository --lib --vcs none
cargo new infra --lib --vcs none
cargo new config --lib --vcs none
```

## メモ:子クレートのCargo.toml
`dependencies`の書き方がworkspace用になる。
```
[dependencies]
# 同じworkspace内の別クレート
aaa = { path = "../aaa" }
bbb = { path = "../bbb" }
ccc= { path = "../ccc" }

# workspace共通クレート
ddd = { workspace = true }
eee = { workspace = true }

# 固有クレート
fff = "0.8"
```
# 例: Viteで「frontend」というディレクトリにプロジェクトを展開する場合



# フロントエンド側(React)
## セットアップ前の準備について
ホスト側にはnpmが入っていない。  
そのため、頑張って一旦Dockerを立ち上げたうえで、コンテナ内で以下を行う。  
Dockerfileやcompose.yamlの`npm run`みたいなやつを全部消せは立ち上げるはずである。  
以下の手順でいい感じにする。
1. Dockerfileの行を以下のように編集する。
```dockerfile
#CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]
CMD ["sleep", "infinity"]
```
2. conpose.devのvolumeは有効にしておく(node_modules)
```yaml
volumes:
  - .:/workspace
  - frontend_node_modules:/workspace/frontend/node_modules
```
3. その状態で、コンテナ内で、後述のセットアップをする
4. Dockerfileを元に戻す

## フロントエンド側のセットアップについて
1. ルートディレクトリを作成する  
```bash
mkdir frontend
```
2. そのディレクトリにプロジェクト展開
```
npm create vite@latest .
```
**選択オプション:**
| 項目 | 選択内容 |
| :--- | :--- |
| **Select a framework** | React |
| **Select a variant** | TypeScript + React + Compiler |
| **Install with npm and start now?** | Yes |

**`.`（カレントディレクトリ）指定による挙動のメモ:**
```
・Project name: コマンドを実行したフォルダ名が自動採用された。(多分)
・Package name: 同上。
・展開場所: そのフォルダ直下にファイル群が展開された。
・補足: プロジェクト名を手動で設定するステップはスキップされた。
```


クローンしたりしたときは初回は`npm install`動作をしなきゃいけなそう？
```
make dev-up
make frontend-shell

npm install
```