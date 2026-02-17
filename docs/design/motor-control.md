# Motor Control プロジェクト設計書

## 概要

TB6612FNGモータードライバを使用して、2つのDCモーターをPWM制御するプロジェクト。
将来的に移動ロボットの制御基盤として拡張することを想定した設計。

## ハードウェア構成

### ターゲットMCU

- **MCU**: STM32F411CEU6 (Black Pill)
- **動作クロック**: 16MHz (内部RC) ※デフォルト設定

### モータードライバ

- **IC**: TB6612FNG (東芝)
- **駆動電圧**: 2.5V〜13.5V
- **最大出力電流**: 1.2A (連続) / 3.2A (ピーク)
- **内蔵機能**: Hブリッジ×2、サーマルシャットダウン

### 接続図

```
STM32F411                TB6612FNG
┌─────────┐              ┌─────────────┐
│     PB6 ├──────────────┤ PWMA        │
│     PB7 ├──────────────┤ PWMB        │
│     PB4 ├──────────────┤ AIN1        │
│     PB5 ├──────────────┤ AIN2        │
│     PB8 ├──────────────┤ BIN1        │
│     PB9 ├──────────────┤ BIN2        │
│     3V3 ├──────────────┤ STBY        │  ※常時有効化
│     GND ├──────────────┤ GND         │
└─────────┘              └─────────────┘
```

**STBY**: 今回はソフトウェア制御せず3V3に直結（常時有効）。
将来的にスリープ制御が必要な場合はGPIOピンに変更可能。

## ピン配置

| ピン | 機能 | 用途 | 備考 |
|------|------|------|------|
| PB6 | TIM4_CH1 (AF2) | PWM-A | モーターA速度制御 |
| PB7 | TIM4_CH2 (AF2) | PWM-B | モーターB速度制御 |
| PB4 | GPIO出力 | AIN1 | モーターA方向制御1 |
| PB5 | GPIO出力 | AIN2 | モーターA方向制御2 |
| PB8 | GPIO出力 | BIN1 | モーターB方向制御1 |
| PB9 | GPIO出力 | BIN2 | モーターB方向制御2 |
| PC13 | GPIO出力 | Status LED | オンボードLED（動作確認用）|

### ピン選定理由

1. **PB4〜PB9の連続配置**: 配線が容易でコネクタ接続に適している
2. **TIM4の活用**: color-ledプロジェクトと同じタイマーで実績あり
3. **I2C1との分離**: 将来的にI2Cセンサー追加時はI2C2（PB10/PB3）を使用可能

## TB6612FNG制御ロジック

### モーターA制御

| AIN1 | AIN2 | PWMA | 動作 |
|------|------|------|------|
| H | L | PWM | 正転（PWMで速度制御）|
| L | H | PWM | 逆転（PWMで速度制御）|
| L | L | - | ストップ（慣性で停止）|
| H | H | - | ブレーキ（短絡制動）|

### モーターB制御

| BIN1 | BIN2 | PWMB | 動作 |
|------|------|------|------|
| H | L | PWM | 正転（PWMで速度制御）|
| L | H | PWM | 逆転（PWMで速度制御）|
| L | L | - | ストップ（慣性で停止）|
| H | H | - | ブレーキ（短絡制動）|

## ソフトウェア設計

### モジュール構成

```
src/
├── main.rs           # エントリポイント、デモ動作
└── motor.rs          # モーター制御モジュール
```

- **motor.rs**: `Motor`, `DualMotor` 構造体と制御ロジックを定義
- **main.rs**: ペリフェラル初期化とデモシーケンスの実行

### 設計方針

embassy-stm32の`SimplePwm`は1つのタイマーインスタンスで複数チャンネルを一括管理する。
そのため、以下のように責務を分離する：

- **`Motor`**: 方向制御（GPIOピン2本）とチャンネル情報のみを保持
- **`DualMotor`**: `SimplePwm`を所有し、2つの`Motor`と協調して速度＋方向を制御

この構成により`SimplePwm`の所有権問題を回避しつつ、
移動ロボット向けの高レベルAPI（`forward`, `turn_left`等）を自然に提供できる。

### Motor構造体

```rust
use embassy_stm32::gpio::Output;
use embassy_stm32::timer::Channel;

/// Motor rotation direction
pub enum Direction {
    Forward,
    Reverse,
    Stop,
    Brake,
}

/// Single motor direction controller for TB6612FNG
///
/// Manages two GPIO pins (IN1/IN2) for direction control.
/// Speed control (PWM) is handled by DualMotor which owns the SimplePwm.
pub struct Motor<'d> {
    in1: Output<'d>,
    in2: Output<'d>,
    channel: Channel,
}

impl<'d> Motor<'d> {
    pub fn new(in1: Output<'d>, in2: Output<'d>, channel: Channel) -> Self;

    /// Set motor direction via IN1/IN2 GPIO pins
    pub fn set_direction(&mut self, direction: Direction);

    /// Get the PWM channel associated with this motor
    pub fn channel(&self) -> Channel;
}
```

### DualMotor構造体

```rust
use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_stm32::pac::TIM4;

/// Dual motor controller for differential drive robot
///
/// Owns the SimplePwm instance and two Motors.
/// Provides both individual motor control and high-level drive commands.
pub struct DualMotor<'d> {
    pwm: SimplePwm<'d, TIM4>,
    motor_a: Motor<'d>,
    motor_b: Motor<'d>,
    max_duty: u32,
}

impl<'d> DualMotor<'d> {
    pub fn new(
        pwm: SimplePwm<'d, TIM4>,
        motor_a: Motor<'d>,
        motor_b: Motor<'d>,
    ) -> Self;

    // --- Individual motor control ---

    /// Set a single motor's direction and speed (0-255)
    pub fn set_motor(&mut self, id: MotorId, direction: Direction, speed: u8);

    // --- High-level drive commands ---

    /// Drive forward at given speed (both motors forward)
    pub fn forward(&mut self, speed: u8);

    /// Drive backward at given speed (both motors reverse)
    pub fn backward(&mut self, speed: u8);

    /// Pivot turn left (motor_a reverse, motor_b forward)
    pub fn turn_left(&mut self, speed: u8);

    /// Pivot turn right (motor_a forward, motor_b reverse)
    pub fn turn_right(&mut self, speed: u8);

    /// Stop both motors (coast)
    pub fn stop(&mut self);

    /// Brake both motors (short brake)
    pub fn brake(&mut self);
}

/// Motor identifier
pub enum MotorId {
    A,
    B,
}
```

### PWM設定

| パラメータ | 値 | 備考 |
|-----------|-----|------|
| タイマー | TIM4 | 汎用タイマー |
| 周波数 | 20kHz | 可聴域外（静音動作）|
| 分解能 | 8bit相当 | 0〜255で速度指定 |
| カウントモード | Edge Aligned Up | 標準的なPWM |

### 速度変換

speed引数（0〜255）からPWMデューティへの変換：

```rust
let duty = speed as u32 * self.max_duty / 255;
```

## デモ動作シーケンス

1. **起動**: Status LED点灯、初期化完了をdefmtでログ出力
2. **前進**: 50%速度で2秒間
3. **停止**: 1秒間
4. **後退**: 50%速度で2秒間
5. **停止**: 1秒間
6. **左旋回**: 50%速度で1秒間
7. **右旋回**: 50%速度で1秒間
8. **ブレーキ**: 完全停止、1秒間
9. **ループ**: 手順2に戻る

各動作の前後でdefmtログを出力し、Status LEDをトグルする。

## 依存クレート

```toml
[dependencies]
embassy-executor = { version = "0.7", features = ["arch-cortex-m", "executor-thread"] }
embassy-stm32 = { version = "0.2", features = ["stm32f411ce", "time-driver-any", "memory-x"] }
embassy-time = { version = "0.4", features = ["tick-hz-32_768"] }
defmt = "0.3"
defmt-rtt = "0.4"
panic-probe = { version = "0.3", features = ["print-defmt"] }
cortex-m = { version = "0.7", features = ["critical-section-single-core"] }
cortex-m-rt = "0.7"
embedded-hal = "0.2"

[profile.dev]
opt-level = 1

[profile.release]
debug = 2
lto = true
opt-level = "s"
```

## embedded-hal バージョンについて

本プロジェクトではcolor-ledプロジェクトと統一して **embedded-hal 0.2** を使用する。

### embedded-hal 0.2 vs 1.0 の主な違い

| 項目 | 0.2 | 1.0 |
|------|-----|-----|
| リリース時期 | 2018年〜 | 2024年1月 安定版リリース |
| エラー型 | 各トレイトで `type Error` を個別定義 | `ErrorType` トレイトに統一 |
| GPIO | `OutputPin`, `InputPin` 等 | `OutputPin`, `InputPin` （fallible only） |
| PWM | `Pwm` トレイト（`get_max_duty`, `set_duty`, `enable` 等） | `SetDutyCycle` トレイト（`set_duty_cycle_fraction` 等） |
| SPI/I2C | 別々のトレイト | 統一的なエラーハンドリング |
| async対応 | なし | `embedded-hal-async` クレートで対応 |
| no_std互換 | あり | あり |

### 0.2を選択した理由

1. **既存プロジェクトとの一貫性**: color-ledプロジェクトが0.2を使用中
2. **embassy-stm32 0.2との組合せ実績**: 動作確認済みの組合せ
3. **PWMトレイト**: `Pwm` トレイト経由で `get_max_duty()` / `set_duty()` を使用（color-ledと同じパターン）

### 将来の移行

embassy-stm32が1.0対応を安定化した段階で、全プロジェクトを一括移行する予定。
移行時の主な変更点：
- `Pwm` トレイト → `SetDutyCycle` トレイトへの置き換え
- `get_max_duty()` / `set_duty()` → `max_duty_cycle()` / `set_duty_cycle()` へ変更
- エラーハンドリングの統一

## ファイル構成

```
stm32-embassy/projects/motor-control/
├── .cargo/
│   └── config.toml       # ビルド設定（probe-rs, linker flags）
├── .gitignore             # /target 除外
├── src/
│   ├── main.rs            # エントリポイント、デモ動作
│   └── motor.rs           # モーター制御モジュール
├── Cargo.toml             # 依存クレート定義
└── rust-toolchain.toml    # Rustツールチェイン (1.92)
```

※ `build.rs` / `memory.x` は不要。embassy-stm32の `memory-x` フィーチャーがメモリレイアウトを自動提供する。

## 電源に関する注意事項

1. **モーター電源**: TB6612FNGのVMは別電源（2.5V〜13.5V）を使用
2. **ロジック電源**: VCCはMCUと共通の3.3Vを使用
3. **GND共通化**: MCUとモータードライバのGNDは必ず接続
4. **デカップリング**: VM, VCC近くに0.1μFコンデンサを配置

## 将来の拡張案

- エンコーダ入力によるフィードバック制御
- I2Cセンサー（IMU等）との連携
- 複数モーター対応（TIM3等の追加タイマー使用）
- UART/BLEによるリモート制御
- `DualMotor` への曲線走行（左右速度差指定）メソッド追加

## 参考資料

- [TB6612FNGデータシート](https://toshiba.semicon-storage.com/jp/semiconductor/product/motor-driver-ics/brushed-dc-motor-driver-ics/detail.TB6612FNG.html)
- [STM32F411リファレンスマニュアル](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411xce-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [Embassy STM32 HAL](https://docs.embassy.dev/embassy-stm32/)
