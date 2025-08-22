#!/bin/bash

# Adaptive Routing - ワンライナーインストールスクリプト (systemd自動選択版)
set -e

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

REPO_URL="https://github.com/NextRouter/adaptiveRouting.git"
INSTALL_DIR="$HOME/adaptiveRouting"

echo -e "${BLUE}🚀 Adaptive Routing ワンライナーインストーラー${NC}"
echo "=============================================="

# Git確認
if ! command -v git >/dev/null 2>&1; then
    echo -e "${YELLOW}📦 Gitをインストール中...${NC}"
    if command -v apt-get >/dev/null 2>&1; then
        sudo apt update && sudo apt install -y git
    elif command -v yum >/dev/null 2>&1; then
        sudo yum install -y git
    else
        echo -e "${RED}❌ Gitを手動でインストールしてください${NC}"
        exit 1
    fi
fi

# リポジトリクローン
if [ -d "$INSTALL_DIR" ]; then
    echo -e "${YELLOW}📁 既存のディレクトリを更新中...${NC}"
    cd "$INSTALL_DIR"
    git pull
else
    echo -e "${YELLOW}📥 リポジトリをクローン中...${NC}"
    git clone "$REPO_URL" "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

# セットアップ実行
echo -e "${GREEN}🛠️ セットアップを開始...${NC}"
chmod +x setup-systemd.sh

# --- ユーザー選択部分を削除し、systemd登録処理を直接実行 ---
echo -e "${GREEN}🔧 systemdサービスとして登録します...${NC}"
# まずビルドのみ実行（バックグラウンドで実行を避ける）
echo -e "${YELLOW}📦 依存関係をインストールしてビルド中...${NC}"

# OS検出
if command -v apt-get >/dev/null 2>&1; then
    sudo apt update
    sudo apt install -y libpcap-dev build-essential curl
elif command -v yum >/dev/null 2>&1; then
    sudo yum groupinstall -y "Development Tools"
    sudo yum install -y libpcap-devel curl
fi

# Rustチェック
if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${YELLOW}🦀 Rustをインストール中...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# ビルド
echo -e "${YELLOW}🔨 プロジェクトをビルド中...${NC}"
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ ビルド完了${NC}"
    echo -e "${YELLOW}🔧 systemdサービスを登録中...${NC}"
    ./setup-systemd.sh
else
    echo -e "${RED}❌ ビルドに失敗しました${NC}"
    exit 1
fi