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

**対処法A（推奨）**: `.cargo/config.toml` の runner に `--connect-under-reset` を追加

```toml
[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32F411CEUx --connect-under-reset"
```

リセット状態を保ちながら接続するため、SWDが一時的に無効化された状態でも確実に回復できる。
embassy-stm32 0.5 では `enable_debug_during_sleep` だけでは再発するケースが確認されているため、こちらを使う。

> **注意**: `--connect-under-reset` を有効にするには、ST-Link の RST ピンと Black Pill の NRST ピンを接続する必要がある（下記「SWD配線」参照）。
> NRSTが未接続の場合、このオプションは機能しない。その場合はBOOT0ボタン操作（対処法「ファームウェアがSWDピンを無効化している」参照）で代替する。

**対処法B**: `enable_debug_during_sleep`を有効にする（embassy-stm32 0.2 では有効だった）

```rust
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // Keep SWD debug pins (PA13/PA14) enabled during sleep.
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

| ST-Link V2 | Black Pill | 備考 |
|-----------|------------|------|
| SWDIO | SWDIO (PA13) | 必須 |
| SWCLK | SWCLK (PA14) | 必須 |
| GND | GND | 必須 |
| 3.3V | 3.3V | 任意（USB給電時は不要）|
| RST | NRST | **追加推奨**（`--connect-under-reset` を有効にする）|

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

#### embassy-stm32 0.5 + embassy-executor 0.9 での追加情報

embassy-stm32 0.5 / embassy-executor 0.9 への移行後に `JtagNoDeviceConnected` が再発した場合の情報。

**アイドル実装の変更（WFI → WFE）**

embassy-executor 0.9 では、アイドル時の割り込み待ち実装が WFI（Wait For Interrupt）から **WFE（Wait For Event）** に変更された。
WFEとDBGMCUの `DBG_SLEEP` ビットとの関係、および WFEがSWDデバッグに与える正確な影響については調査中。

**`enable_debug_during_sleep` について**

`enable_debug_during_sleep = true` は embassy-stm32 0.2 では有効だったが、0.5 では再発するケースが確認されている。
0.5 での動作については調査中。

**probe-rs のバージョン確認**

```bash
probe-rs --version
```

> **注意**: `probe-rs -v` や `probe-rs --verbose` はバージョン表示ではなく verbose フラグのため使用しない。

**現時点の推奨ワークアラウンド**

1. ST-Link の RST ピンと Black Pill の NRST ピンを接続する
2. `.cargo/config.toml` で `--connect-under-reset` を使用する（対処法A参照）

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
