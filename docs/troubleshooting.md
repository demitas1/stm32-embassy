# Troubleshooting

## ST-Link/probe-rs接続の問題

### JtagNoDeviceConnected エラー

```
WARN probe_rs::probe::stlink: send_jtag_command 242 failed: JtagNoDeviceConnected
Error: Connecting to the chip was unsuccessful.
```

#### 症状

- `probe-rs list` でST-Linkは認識されている
- しかし `cargo run --release` でターゲットに接続できない

#### 診断

OpenOCDで詳細を確認：

```bash
docker compose run --rm embassy-dev bash -c "timeout 10 openocd -f interface/stlink.cfg -f target/stm32f4x.cfg -c 'init' -c 'exit' 2>&1"
```

以下のような出力が得られる場合：
```
Info : Target voltage: 3.26V  # ← 電圧は正常
Warn : target stm32f4x.cpu examination failed  # ← CPU通信失敗
```

#### 原因と対処

##### 1. Sleepモード中にデバッグが無効化されている

Embassyのデフォルト設定では、sleepモード中にSWDデバッグが無効化される場合がある。

**症状**:
- 初回の `cargo run --release` は成功する
- 2回目以降で `JtagNoDeviceConnected` エラーが発生
- `probe-rs download` + `probe-rs reset` では正常にフラッシュできる

**対処法**: `enable_debug_during_sleep`を有効にする

```rust
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // Keep SWD debug pins (PA13/PA14) enabled during sleep.
    // Without this, probe-rs cannot reconnect after the first flash.
    config.enable_debug_during_sleep = true;
    let p = embassy_stm32::init(config);
    // ...
}
```

##### 2. ファームウェアがSWDピンを無効化している

以前書き込んだファームウェアがPA13（SWDIO）/PA14（SWCLK）を別の用途（GPIO等）に設定していると、SWDが無効になる。

**対処法**: BOOT0ボタンを使ってリカバリーモードで起動

1. Black PillのBOOT0ボタンを**押しながら**
2. RESETボタンを押して離す
3. BOOT0を離す
4. この状態で `cargo run --release` を実行

**予防策**: PA13/PA14をGPIOとして使用しない

```rust
// Embassy: PA13/PA14は使用しない
// let pa13 = p.PA13; // ← SWDIOなので避ける
// let pa14 = p.PA14; // ← SWCLKなので避ける
```

##### 3. SWD配線の問題

ST-Link V2とBlack Pill間の接続を確認：

| ST-Link V2 | Black Pill |
|-----------|------------|
| SWDIO | SWDIO (PA13) |
| SWCLK | SWCLK (PA14) |
| GND | GND |
| 3.3V | 3.3V (任意) |

確認ポイント：
- ジャンパーワイヤーの断線
- SWDIOとSWCLKが逆になっていないか
- はんだ不良

##### 4. ターゲットボードの電源問題

OpenOCDの出力で`Target voltage: 0.0V`と表示される場合は、ボードに電源が供給されていない。

- USBケーブルでBlack Pillに給電
- または、ST-Linkの3.3Vピンから給電

##### 5. ハードウェア故障

上記すべてを確認しても解決しない場合：
- 別のST-Linkで試す
- 別のBlack Pillボードで試す

---

### probe-rsがST-Linkを認識しない

```
Error: No probe was found.
```

#### 対処

1. ST-LinkのUSBケーブルを抜き差し
2. udevルールの確認：
   ```bash
   cat /etc/udev/rules.d/99-stlink.rules
   ```
3. Dockerコンテナを再起動：
   ```bash
   docker compose down
   docker compose run --rm embassy-dev
   ```

---

## ビルドエラー

### linking with `flip-link` failed

```
error: linking with `flip-link` failed
```

flip-linkがインストールされていない。

```bash
cargo install flip-link --locked
```

---

### Permission denied (cargo build時)

Cargoキャッシュの権限問題。ボリュームを削除して再ビルド：

```bash
docker compose down -v
docker compose build --no-cache
```

---

## ログの問題

### defmtログが表示されない

`.cargo/config.toml`で`DEFMT_LOG`環境変数を確認：

```toml
[env]
DEFMT_LOG = "debug"
```

設定可能なレベル: `trace`, `debug`, `info`, `warn`, `error`
