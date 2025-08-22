# adaptive-routing

ネットワークのパケットロス率を監視し、品質が低下した際に自動でネットワーク経路（WAN）を切り替えるツールです。Prometheusからメトリクスを取得し、nftablesを操作してルーティングを制御します。

## 🚀 クイックスタート

### ワンコマンドで実行

```bash
curl -sSL https://raw.githubusercontent.com/NextRouter/adaptiveRouting/main/install.sh | bash
```

### 手動インストールと実行

```bash
# リポジトリをクローン
git clone https://github.com/NextRouter/adaptiveRouting.git
cd adaptiveRouting

# ビルド
cargo build --release

# 実行 (sudo権限が必要です)
sudo ./target/release/adaptiveRouting
```

## 📋 必要要件

- Linux OS
- Rust 1.70+
- `sudo`権限 (`nft`コマンド実行のため)
- `nftables`がインストールされ、設定済みであること
- `localpacketdump`のようなパケット監視ツールがPrometheusメトリクスを`http://localhost:9090`で提供していること

## 🔧 機能

- **パケットロス監視**: PrometheusからIP別のパケットロス関連メトリクスを定期的に取得します。
- **自動経路切り替え**: パケットロスが閾値を超えたIPアドレスの通信経路を、設定された他のWANインターフェース（例: `wan1` <-> `wan2`）に自動で切り替えます。
- **nftables連携**: `nftables`のsetを操作して、IPアドレスのルーティングルールを動的に変更します。

## 🛠️ 手動ビルド

```bash
# 依存関係インストール (Ubuntu/Debian)
sudo apt update
sudo apt install -y build-essential

# Rustインストール (未インストールの場合)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# ビルド & 実行
cargo build --release
sudo ./target/release/adaptiveRouting
```

## 🔧 systemdサービスとして登録

サービスの自動起動を設定すると便利です。

### 1. サービスファイル作成

以下のコマンドを実行して、`systemd`サービスファイルを作成します。

```bash
# `adaptiveRouting`ディレクトリの絶対パスを取得
WORKDIR=$(pwd)

# サービスファイルを作成
sudo tee /etc/systemd/system/adaptiverouting.service > /dev/null <<EOF
[Unit]
Description=Adaptive Routing Service
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
Group=root
ExecStart=$WORKDIR/target/release/adaptiveRouting
WorkingDirectory=$WORKDIR
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# セキュリティ設定
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/tmp

[Install]
WantedBy=multi-user.target
EOF
```

### 2. サービス有効化・開始

```bash
# systemd設定リロード
sudo systemctl daemon-reload

# サービス有効化（起動時自動開始）
sudo systemctl enable adaptiverouting.service

# サービス開始
sudo systemctl start adaptiverouting.service
```

### 3. サービス管理コマンド

```bash
# ステータス確認
sudo systemctl status adaptiverouting.service

# ログ確認
sudo journalctl -u adaptiverouting.service -f

# サービス停止
sudo systemctl stop adaptiverouting.service

# サービス再起動
sudo systemctl restart adaptiverouting.service

# 自動起動無効化
sudo systemctl disable adaptiverouting.service

# サービス削除
sudo systemctl stop adaptiverouting.service
sudo systemctl disable adaptiverouting.service
sudo rm /etc/systemd/system/adaptiverouting.service
sudo systemctl daemon-reload
```

## ⚠️ 注意事項

- `nftables`の操作には`root`権限が必要です。
- このツールは`http://localhost:9090`で動作するPrometheusエンドポイントに依存します。`localpacketdump`などのメトリクス提供元が正しく設定・実行されていることを確認してください。
- `nftables`の設定が適切に行われている必要があります。特に、`wan1_hosts`と`wan2_hosts`という名前のsetが`inet mangle`テーブルに存在することを前提としています。

## 📝 ライセンス

MIT License
