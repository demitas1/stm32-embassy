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

##### 1. Sleepモード中にデバッグが無効化されている（ST-Link V2クローン）

embassy-executor 0.9 のアイドル実装（WFE）により、スリープ中にSWDデバッグが切断される。
ST-Link V2クローンでは根本的な回避策はない（詳細は下記「追加情報」参照）。

**症状**:
- 初回の `cargo run --release` は成功する
- 2回目以降で `JtagNoDeviceConnected` エラーが発生

**確実な対処法（BOOT0）**: 書き込み前にBOOT0ボタンでリカバリーモードに入る

1. Black PillのBOOT0ボタンを**押しながら**
2. RESETボタンを押して離す
3. BOOT0を離す
4. この状態で `cargo run --release` を実行

**`enable_debug_during_sleep` について**（効果なし）

```rust
config.enable_debug_during_sleep = true;  // DBGMCU_CR = 0x7 に設定される
```

DBGMCU_CR が正しく 0x7 に設定されても、ST-Link V2クローンでは接続不可。
SCR.SLEEPDEEP=0（通常Sleepモード）でも同様に失敗することを確認済み。

**`--connect-under-reset` について**（ST-Link V2クローンでは効果なし）

```toml
runner = "probe-rs run --chip STM32F411CEUx --connect-under-reset"
```

ST-Link V2クローンは `hla_swd` トランスポートの制限により、NRSTを保持したまま
接続する「真のconnect-under-reset」が実現できない（`connect_deassert_srst`）。
物理的にNRSTを配線しても、クローンのRSTピンがターゲットをドライブしないケースがある。

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

#### embassy-stm32 0.5 + embassy-executor 0.9 + ST-Link V2クローン：調査結果

**根本原因（確定）**

embassy-executor 0.9 のアイドル実装が WFI → **WFE** に変更された。
STM32F411 では WFE スリープ中に ST-Link V2クローン経由の SWD AP アクセスが完全に切断される。

確認済みの事実：
- `DBGMCU_CR = 0x7`（DBG_SLEEP | DBG_STOP | DBG_STANDBY）を設定しても接続不可
- `SCR.SLEEPDEEP = 0`（STOPモードではなく通常Sleepモード）でも接続不可
- `probe-rs attach`（リセットなし直接接続）も失敗
- SWD速度を 100kHz に下げても変化なし
- ST-Link V2クローンの RST ピンはターゲット NRST をドライブしない（リセット不可）

**ST-Link V2クローンの制限**

- `hla_swd` トランスポートは真の connect-under-reset 非対応（`connect_deassert_srst`）
- probe-rs: `Custom reset sequences are not supported on ST-Link V2.`
- OpenOCD: `connect_deassert_srst`（接続前にNRSTを解除してしまう）

**バスマトリクス無効化：DBGMCU_CR が効かない理由**

[Jeff McBride, "Errors using RTT and WFI" (2025)](https://jeffmcbride.net/blog/2025/05/22/rtt-errors-with-wfi/) より：

> `DBGMCU_CR.DBG_SLEEP=1` を設定してもデバッグコアのクロックは維持されるが、
> **AHB バスマトリクスは「アクティブなバスマスター」が存在しない場合に無効化される。
> デバッグコア自体はアクティブマスターとしてカウントされない。**

WFE でCPUがスリープ → AHB バスのアクティブマスターが消える → バスマトリクス無効化
→ SWD AHB-AP が SRAM/周辺機器にアクセス不能 → `JtagNoDeviceConnected`

**DMA1 クロック有効化（STM32F411 では効果なし・確認済み）**

Jeff McBride のブログでは STM32G0 での DMA1 有効化が有効とされているが、
STM32F411 では RCC_AHB1ENR.DMA1EN ビットを立てるだけでは効果なかった。

理由：クロックを有効にするだけでは DMA1 はバスマスターにならない。
AHBバスマトリクスにアクティブマスターとして認識されるには実際に DMA 転送が実行中である必要がある。
継続的な DMA 転送を仕掛ける方法も理論上は可能だが、実装の複雑さに見合わない。

**関連 Issue・参考リンク**

| リポジトリ | リンク | 内容 |
|-----------|--------|------|
| probe-rs/probe-rs | [#350](https://github.com/probe-rs/probe-rs/issues/350) | WFI/WFE でSTM32のデバッグ・フラッシュが不安定（主要トラッカー、未解決）|
| probe-rs/probe-rs | [#3715](https://github.com/probe-rs/probe-rs/issues/3715) | Embassy + await で `JtagNoDeviceConnected`。await 除去で再現しないことを確認 |
| knurling-rs/probe-run | [#85](https://github.com/knurling-rs/probe-run/issues/85) | STM32F411 + WFI で接続不可 → `--connect-under-reset` オプション追加のきっかけ |
| Jeff McBride blog | [2025-05-22](https://jeffmcbride.net/blog/2025/05/22/rtt-errors-with-wfi/) | バスマトリクス無効化の仕組みとDMA1回避策 |
| cliffle.com | [STM32 WFI bug](https://cliffle.com/blog/stm32-wfi-bug/) | STM32 WFI による命令パイプライン汚染の詳細解析 |

**結論と解決策**

| 方法 | コスト | 効果 |
|------|--------|------|
| BOOT0ボタン操作（毎回） | ゼロ | 確実（ST-Link V2 使用時の対策）|
| DMA1 クロック有効化 | ゼロ | **効果なし**（STM32F411 で確認済み）|
| picoprobe (CMSIS-DAP) に乗り換え | Pico本体のみ | **恒久解決（確認済み）** NRST 不要 |
| J-Link EDU Mini 等に乗り換え | 約3000円 | 恒久解決（未検証）|

**解決策：picoprobe (CMSIS-DAP) への切り替え（確認済み）**

Raspberry Pi Pico に [picoprobe](https://github.com/raspberrypi/picoprobe) ファームウェアを書き込み
CMSIS-DAP 対応プローブとして使用することで、BOOT0 操作なしで `cargo run --release` が動作することを確認済み。

確認済みの構成：
- プローブ: Raspberry Pi Debugprobe on Pico (USB `2e8a:000c`)
- NRST 接続: **不要**（4線 SWD のみで動作）
- `--connect-under-reset` オプション: **不要**
- `.cargo/config.toml` の runner: `probe-rs run --chip STM32F411CEUx --probe 2e8a:000c`

**ST-Link V2 で解決できない根本理由**

ST-Link V2 は HLA（High Level Adapter）トランスポートを使用する。
probe-rs は高レベルコマンドを ST-Link に送るだけで、SWD の細かいタイミング制御は
ST-Link 内部ファームウェアが担う。このため、WFE スリープ中の SWD AP アクセスが
ブロックされても probe-rs 側で対処できない。

probe-rs のログにもこの制限が明記されている：
```
Custom reset sequences are not supported on ST-Link V2.
Falling back to standard probe reset.
```

OpenOCD でも同様に `connect_deassert_srst`（接続前に NRST を解放）が強制される。
これは ST-Link V2 の設計上の制限であり、ファームウェアアップデートでも解消されない。

**CMSIS-DAP で解決できた理由**

CMSIS-DAP では probe-rs が SWD のビットレベル操作を直接制御できる。
WFE スリープ中でも SWD ライン・リセットシーケンスで DP を再初期化し、
AHB-AP への接続を確立できる。NRST によるチップリセットは不要。

**probe-rs のバージョン確認**

```bash
probe-rs --version
```

> **注意**: `probe-rs -v` や `probe-rs --verbose` はバージョン表示ではなく verbose フラグのため使用しない。

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
