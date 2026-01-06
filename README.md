# STM32F4 Embassy Development Environment

STM32F411CE (Black Pill) + Embassy (Rust) + Docker開発環境

## ディレクトリ構成

```
stm32/
├── stm32-deps/           # 共通依存ライブラリ (FreeRTOS用、本環境では未使用)
├── stm32-freertos/       # FreeRTOS環境 (C/C++)
└── stm32-embassy/        # このリポジトリ
    ├── Dockerfile
    ├── docker-compose.yml
    └── projects/
        └── led-blink/    # サンプルプロジェクト
```

## 動作確認済み環境

- **Rust**: 1.92
- **probe-rs-tools**: 0.30.0
- **Target**: STM32F411CEU6 (Black Pill)

## ホスト側の設定

### udevルール (ST-Link/probe-rs用)

ST-Linkをroot権限なしで使用するため、udevルールを設定する。

```bash
sudo tee /etc/udev/rules.d/99-stlink.rules << 'EOF'
# ST-Link V2
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", MODE="0666", GROUP="plugdev"
# ST-Link V2-1
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", MODE="0666", GROUP="plugdev"
# ST-Link V3
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374e", MODE="0666", GROUP="plugdev"
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374f", MODE="0666", GROUP="plugdev"
EOF

sudo udevadm control --reload-rules
sudo udevadm trigger
```

### plugdevグループへの追加

```bash
# 現在のユーザーがplugdevに所属しているか確認
groups | grep plugdev

# 所属していない場合は追加（要ログアウト/ログイン）
sudo usermod -aG plugdev $USER
```

## セットアップ

### Dockerイメージのビルド

```bash
cd stm32-embassy
docker compose build
```

初回ビルドにはprobe-rs-tools等のインストールで時間がかかります（5-10分程度）。

### UID/GIDのカスタマイズ

ホストユーザーのUID/GIDが1000以外の場合：

```bash
UID=$(id -u) GID=$(id -g) docker compose build
```

## 使い方

### ビルド

```bash
# コンテナに入る
docker compose run --rm embassy-dev

# プロジェクトディレクトリへ移動
cd led-blink

# ビルド
cargo build --release

# クリーンビルド
cargo clean && cargo build --release
```

### フラッシュ

ST-Linkを接続した状態で:

```bash
# probe-rsを使用（ビルド＆フラッシュ＆実行）
cargo run --release
```

### ワンライナー

```bash
# コンテナ外からビルド
docker compose run --rm embassy-dev bash -c "cd led-blink && cargo build --release"

# コンテナ外からフラッシュ
docker compose run --rm embassy-dev bash -c "cd led-blink && cargo run --release"
```

### defmtログの確認

`cargo run`実行時にdefmtログがRTT経由で表示されます。

## ターゲットハードウェア

- **MCU**: STM32F411CEU6 (Black Pill)
- **クロック**: 設定可能（デフォルトは内部RC）
- **Flash**: 512KB
- **RAM**: 128KB
- **LED**: PC13 (Active Low)

## Embassy設定

### 主な依存クレート

| クレート | バージョン | 用途 |
|---------|-----------|------|
| embassy-executor | 0.7 | 非同期タスクエグゼキュータ |
| embassy-time | 0.4 | 時間管理、Timer |
| embassy-stm32 | 0.2 | STM32 HAL |
| defmt | 0.3 | 組み込み向けログ |
| probe-rs-tools | 0.30 | デバッガ/フラッシュツール |

### プロファイル設定

- **dev**: opt-level = 1 (デバッグ用、最小限の最適化)
- **release**: LTO有効、サイズ最適化、デバッグ情報保持

## プロジェクト

### led-blink

基本的なLED点滅サンプル。PC13のオンボードLEDを500ms間隔で点滅させる。

Embassy非同期ランタイムを使用した最小構成のサンプル。

## 新規プロジェクト作成

```bash
# projects/ディレクトリ内に新しいプロジェクトを作成
cd projects
cargo new my-project --name my-project

# led-blinkから設定ファイルをコピー
cp -r led-blink/.cargo my-project/
cp led-blink/rust-toolchain.toml my-project/

# Cargo.tomlの依存関係を編集
```

## トラブルシューティング

### probe-rsが接続できない

1. ST-Linkが正しく接続されているか確認
2. udevルールが設定されているか確認
3. コンテナがprivilegedモードで動作しているか確認

### Permission denied エラー (cargo build時)

Cargoキャッシュの権限問題。ボリュームを削除して再ビルド：

```bash
docker compose down -v
docker compose build --no-cache
```

### ビルドエラー: linking with `flip-link` failed

flip-linkがインストールされていない場合：
```bash
cargo install flip-link --locked
```

### defmtログが表示されない

`.cargo/config.toml`で`DEFMT_LOG`環境変数が設定されているか確認。

## Docker環境の詳細

- コンテナ内では`developer`ユーザー（UID/GID: 1000）として実行
- Cargoキャッシュは名前付きボリュームで永続化
- `/projects`ディレクトリがホストの`./projects`にマウント
