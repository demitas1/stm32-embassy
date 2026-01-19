# Color LED プロジェクト設計書

## 概要

RGB LEDを使用してレインボーエフェクト（虹色グラデーション）を表示するプロジェクト。
TIM4のハードウェアPWMで3チャンネルを同時制御し、HSV色空間からRGBへの変換により滑らかな色遷移を実現。

## ハードウェア構成

### ターゲットMCU

- **MCU**: STM32F411CEU6 (Black Pill)
- **動作クロック**: 16MHz (内部RC) ※デフォルト設定

### RGB LED

- **種別**: コモンカソード型RGB LED
- **駆動方式**: アクティブハイ（GPIO Highで点灯）
- **電流制限**: 各色に適切な抵抗を接続（例: 220Ω〜330Ω）

### 接続図

```
STM32F411                RGB LED (Common Cathode)
┌─────────┐              ┌─────────────┐
│     PB6 ├──[R]─────────┤ Red Anode   │
│     PB7 ├──[R]─────────┤ Green Anode │
│     PB8 ├──[R]─────────┤ Blue Anode  │
│     GND ├──────────────┤ Cathode     │
└─────────┘              └─────────────┘
                         [R] = 220Ω〜330Ω
```

## ピン配置

| ピン | 機能 | 用途 | 備考 |
|------|------|------|------|
| PB6 | TIM4_CH1 (AF2) | Red LED | PWM制御 |
| PB7 | TIM4_CH2 (AF2) | Green LED | PWM制御 |
| PB8 | TIM4_CH3 (AF2) | Blue LED | PWM制御 |
| PC13 | GPIO出力 | Status LED | オンボードLED（動作確認用）|

### ピン選定理由

1. **TIM4の3チャンネル連続使用**: 同一タイマーで同期したPWM出力
2. **PB6〜PB8の連続配置**: 配線が容易
3. **TIM4_CH4（PB9）未使用**: 将来の拡張用に確保

## PWM設定

| パラメータ | 値 | 備考 |
|-----------|-----|------|
| タイマー | TIM4 | 汎用タイマー |
| 周波数 | 1kHz | LED制御に十分な周波数 |
| 分解能 | 16bit | SimplePwmのmax_duty値に依存 |
| カウントモード | Edge Aligned Up | 標準的なPWM |
| 出力タイプ | Push-Pull | アクティブハイ駆動 |

## ソフトウェア設計

### モジュール構成

```
src/
└── main.rs           # エントリポイント、全機能を含む
```

### 色空間変換

HSV（色相・彩度・明度）からRGB（赤・緑・青）への変換を実装。

```rust
/// Convert HSV to RGB
///
/// # Arguments
/// * `hue` - Hue value (0-359)
/// * `sat` - Saturation (0-255)
/// * `val` - Value/Brightness (0-255)
///
/// # Returns
/// (r, g, b) tuple with values 0-255
fn hsv_to_rgb(hue: u16, sat: u8, val: u8) -> (u8, u8, u8);
```

### HSV色空間

| パラメータ | 範囲 | 説明 |
|-----------|------|------|
| Hue（色相） | 0〜359 | 色の種類（赤→黄→緑→シアン→青→マゼンタ→赤）|
| Saturation（彩度） | 0〜255 | 色の鮮やかさ（0=灰色、255=純色）|
| Value（明度） | 0〜255 | 明るさ（0=黒、255=最大輝度）|

### レインボーエフェクト

Hue値を0から359まで連続的にインクリメントすることで、虹色のグラデーションを生成。

```
Hue=0   → 赤
Hue=60  → 黄
Hue=120 → 緑
Hue=180 → シアン
Hue=240 → 青
Hue=300 → マゼンタ
Hue=359 → 赤（ループ）
```

## 動作シーケンス

1. **初期化**
   - SWDデバッグ有効化設定
   - GPIO/タイマー初期化
   - PC13 Status LED: High（消灯）
   - defmtで起動ログ出力

2. **メインループ（50ms周期）**
   - HSV→RGB変換（sat=255, val=255固定）
   - PWMデューティ比設定（0-255を0-max_dutyにスケーリング）
   - Hue値インクリメント（0〜359でループ）
   - 20回ごとにStatus LED点滅 + ログ出力

### タイミング

| 項目 | 値 | 備考 |
|------|-----|------|
| 更新周期 | 50ms | 滑らかな色遷移 |
| 1周期 | 18秒 | 360色 × 50ms |
| Status LED | 1秒周期 | 20 × 50ms |

## 依存クレート

```toml
[dependencies]
embassy-executor = { version = "0.7", features = ["arch-cortex-m", "executor-thread"] }
embassy-stm32 = { version = "0.2", features = [
    "stm32f411ce",
    "time-driver-any",
    "memory-x",
]}
embassy-time = { version = "0.4", features = ["tick-hz-32_768"] }
embedded-hal = "0.2"
defmt = "0.3"
defmt-rtt = "0.4"
panic-probe = { version = "0.3", features = ["print-defmt"] }
cortex-m = { version = "0.7", features = ["critical-section-single-core"] }
cortex-m-rt = "0.7"
```

## ファイル構成

```
stm32-embassy/projects/color-led/
├── .cargo/
│   └── config.toml       # ビルド設定（ターゲット、runner）
├── src/
│   └── main.rs           # メインプログラム
├── Cargo.toml            # 依存クレート定義
├── Cargo.lock            # 依存バージョンロック
└── rust-toolchain.toml   # Rustツールチェイン（1.92）
```

## ビルド・実行

```bash
# Dockerコンテナ起動
cd stm32-embassy
docker compose run --rm embassy-dev

# コンテナ内でビルド
cd color-led
cargo build --release

# フラッシュ＆実行（ST-Link接続時）
cargo run --release
```

## デバッグ出力例

```
INFO  Embassy STM32F4 Color LED Rainbow started!
INFO  PWM max duty cycle: 15999
INFO  Hue: 20, RGB: (255, 85, 0)
INFO  Hue: 40, RGB: (255, 170, 0)
INFO  Hue: 60, RGB: (255, 255, 0)
...
```

## 注意事項

### SWDデバッグの維持

```rust
config.enable_debug_during_sleep = true;
```

この設定がないと、probe-rsでの再接続時に「JtagNoDeviceConnected」エラーが発生する。

### LED電流制限

RGB LEDの各色に適切な電流制限抵抗を接続すること。
STM32のGPIOは最大25mA出力のため、LED直結は推奨しない。

## 回路例

```
VCC (3.3V) ─────┬─────┬─────┐
                │     │     │
              [220Ω][220Ω][220Ω]
                │     │     │
                R     G     B  ← RGB LED Anodes
                └──┬──┴──┬──┘
                   │     │
                Common Cathode
                   │
                  GND
```

※上記はVCC駆動の場合。本プロジェクトはGPIO駆動のため以下の構成：

```
PB6 ──[220Ω]── Red Anode    ─┐
PB7 ──[220Ω]── Green Anode  ─┼─ Common Cathode ── GND
PB8 ──[220Ω]── Blue Anode   ─┘
```

## 将来の拡張案

- WS2812B（NeoPixel）対応
- 複数パターン切り替え（ボタン入力）
- 明度・彩度の動的変更
- 音楽連動（ADC入力）

## 参考資料

- [STM32F411リファレンスマニュアル](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411xce-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [Embassy STM32 HAL - SimplePwm](https://docs.embassy.dev/embassy-stm32/git/stm32f411ce/timer/simple_pwm/struct.SimplePwm.html)
- [HSV色空間 - Wikipedia](https://ja.wikipedia.org/wiki/HSV%E8%89%B2%E7%A9%BA%E9%96%93)
