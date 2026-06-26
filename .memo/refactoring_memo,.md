# リファクタリングメモ
現在、backendが完成したところである。
フロントエンド作成に向け、一旦リファクタリングを行おうと思う。
その点をメモしていく。
実際にすべてをリファクタリングするかは要検討である

## Nodeが中心であることをもっと設計書に書いてもいい
YaakoDriveはFile管理システムというよりNode管理システムといっていい可能性がある

## Upload整合性
後でチャンクアップロードを作るならUpload Sessionという概念がほぼ確実に出る。なのでtmpではなくupload_sessionという言葉を今から使ってもいいかもしれない

## ストレージ抽象
今はLocal Storageですが、設計書にStorage Serviceという言葉を少し書いておくと、将来
```
Local
↓
S3
↓
NAS
```
へ差し替えられるのではないか。

## Dashboard
MVPに使用容量があります。気になったのは毎回SUMするの？です。例えばSELECT SUM(size)でも十分ですが、将来数百万ファイルになると重いです。将来的にはUserStatisticsのような集計テーブルを導入する可能性があります。MVPでは不要ですが、設計思想としてDashboardは集計系という認識を持っておくと後で楽

## Search
検索は後から意外と大変になります。例えばILIKE '%abc%'なのかpg_trgmなのか全文検索なのか。MVPでは十分ですが、将来PostgreSQL Full Text Searchまで考えるならRepositoryを少し意識すると後で楽になる。
最終的には部分一致のほかに、少しでも一致(打ち間違えなど)で、一致している順に出すのようなこともしたいと考えている。
`例: 検索(errr) 検索結果(error)`










## backend/Cargo.toml
tokioがfullになっているので最終的には削ってもいいかもしれない。

## Nodeクレート
### 責務の考え
Nodeに振る舞いが無い。
今は`pub name: String`になっているが、Renameはユースケースがやりそうである。
`node.rename(...)`に寄せてもいいのではないか。
こうするとRename RuleがNodeへ閉じ込められるのではないか。
同じように
```rust
delete()
restore()
move_to()
```
などもNode側へ持たせた方がいいのではないか。Name(String)というValue Objectにするのも良いかもしれない。


### エラー型
```rust
impl TryFrom<&str> for NodeStatus {
  type Error = String;
  ...
}
```
ここをStringではなくNodeErrorでもいいかもしれない

### NodeError
これは少し違和感があるかもしれない。例えばAlreadyDeletedがあります。でもNode自身にdelete()がありません。つまりエラーだけある。これはUseCaseにロジックがある可能性があります。私はNodeErrorはNodeのメソッドから返ってきてほしいかもしれない。


## NodeRepository
### 肥大化の初期衝動について
DashboardQueryServiceやQueryServiceを分けて、別々に管理してもいいかもしれない。
RepositoryがNodeRepositoryという名前なのにやっている仕事が
```
CRUD
検索
Dashboard
Tree
Trash
Restore
```
のように全部になっている。つまりAggregate RepositoryではなくNodeに関する全DBアクセス窓口になっています。






# 特に着手したいリファクタリング点
- もう少しNodeのドメインロジックを育てる → やった
- NodeRepositoryをDashboardQueryServiceやQueryServiceに分ける(UseCase中心になってると思います。そのため、Query系（Dashboard・Search・Trash一覧など）をRepositoryから切り出す処理はこの後リファクタリングしようと思う)
- まだ通常ユーザを作成するユースケースなくね？










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