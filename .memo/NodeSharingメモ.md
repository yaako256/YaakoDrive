# NodeSharingメモ.md
友人への共有はTailscaleのNodeSharingというものを使って行う。
これは`tailnetに招待(無料枠の人数制限あり)`とは異なり、デバイス単位でtailscaleのIPアドレスを共有するものらしい。人数制限はない。
NodeShareingでデバイス単位でtailscaleのIPアドレスを共有したうえで、Access consoleで共有するポートを絞る感じだと思われる。
これをマイクラサーバなどでも使う。


## ACLでの設定
`Tailscaleの管理画面→Access Controls`に行く。
詳細設定をするため、`Visual editor`ではなく`Json editor`を表示する。
デフォルトでは以下のような設定が書かれている。
```json
// Example/default ACLs for unrestricted connections.
{
	// Declare static groups of users. Use autogroups for all users or users with a specific role.
	// "groups": {
	//  	"group:example": ["alice@example.com", "bob@example.com"],
	// },

	// Define the tags which can be applied to devices and by which users.
	// "tagOwners": {
	//  	"tag:example": ["autogroup:admin"],
	// },

	// Define grants that govern access for users, groups, autogroups, tags,
	// Tailscale IP addresses, and subnet ranges.
	"grants": [
		// Allow all connections.
		// Comment this section out if you want to define specific restrictions.
		{"src": ["*"], "dst": ["*"], "ip": ["*"]},

		// Allow users in "group:example" to access "tag:example", but only from
		// devices that are running macOS and have enabled Tailscale client auto-updating.
		// {"src": ["group:example"], "dst": ["tag:example"], "ip": ["*"], "srcPosture":["posture:autoUpdateMac"]},
	],

	// Define postures that will be applied to all rules without any specific
	// srcPosture definition.
	// "defaultSrcPosture": [
	//      "posture:anyMac",
	// ],

	// Define device posture rules requiring devices to meet
	// certain criteria to access parts of your system.
	// "postures": {
	//      // Require devices running macOS, a stable Tailscale
	//      // version and auto update enabled for Tailscale.
	// 	"posture:autoUpdateMac": [
	// 	    "node:os == 'macos'",
	// 	    "node:tsReleaseTrack == 'stable'",
	// 	    "node:tsAutoUpdate",
	// 	],
	//      // Require devices running macOS and a stable
	//      // Tailscale version.
	// 	"posture:anyMac": [
	// 	    "node:os == 'macos'",
	// 	    "node:tsReleaseTrack == 'stable'",
	// 	],
	// },

	// Define users and devices that can use Tailscale SSH.
	"ssh": [
		// Allow all users to SSH into their own devices in check mode.
		// Comment this section out if you want to define specific restrictions.
		{
			"action": "check",
			"src":    ["autogroup:member"],
			"dst":    ["autogroup:self"],
			"users":  ["autogroup:nonroot", "root"],
		},
	],

	// Test access rules every time they're saved.
	// "tests": [
	//  	{
	//  		"src": "alice@example.com",
	//  		"accept": ["tag:example"],
	//  		"deny": ["100.101.102.103:443"],
	//  	},
	// ],
}
```
基本は解説であるが、有効にされている部分だけピックアップする。
```json
{
	"grants": [
    // 自分自身のフルアクセス許可
		{"src": ["*"], "dst": ["*"], "ip": ["*"]},
	],

	// SSHの追加機能設定
  // ざっくりいうと、自分自身の端末同士だったら安全にSSHログインしていいよというもの
	"ssh": [
		{
			"action": "check",
			"src":    ["autogroup:member"],
			"dst":    ["autogroup:self"],
			"users":  ["autogroup:nonroot", "root"],
		},
	],
}
```
この部分に、新たにYaakoDriveの設定を追加する。  
(コメント部分は省略)
```json
{
	"grants": [
    	// 自分自身のフルアクセス許可
		{"src": ["*"], "dst": ["*"], "ip": ["*"]},

    // 追加
    // NodeSharingでの共有
    {
      // 扱い: NodeShaingの人だけ
      "src":    ["autogroup:shared"],
      // 対象のIP
      "dst": ["<自宅サーバのTailscale-IP>"],
      // 対象のポート。3000番(YaakoDrive)を許可
      "ip":  ["3000"]
    },
	],

	// SSHの追加機能設定
	"ssh": [
		{
			"action": "check",
			"src":    ["autogroup:member"],
			"dst":    ["autogroup:self"],
			"users":  ["autogroup:nonroot", "root"],
		},
	],
}
```
これで、YaakoDriveの3000番だけ、共有ユーザに許可された状態となる。

## 友人へのShare
`管理画面→Machines`で、自宅サーバPC棟の3点リーダから`Share...`を選択する。  
`Copy share link`でリンクをコピーし、友人に送り付ける。  
友人がそのリンクを踏み、Tailscaleで表示されていたら完了。  
ちなみにこの2つにはこのような違いがある。
```md
# 1回限り有効の共有URL
Copy share link

# 何人でも何回でも使い回せる共有URL
Copy reusable share link
```

