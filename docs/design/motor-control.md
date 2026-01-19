# Motor Control プロジェクト設計書

## 概要

TB6612FNGモータードライバを使用して、2つのDCモーターをPWM制御するプロジェクト。

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
└── motor.rs          # モーター制御モジュール（オプション）
```

### Motor構造体設計

```rust
/// Motor direction
pub enum Direction {
    Forward,
    Reverse,
    Stop,
    Brake,
}

/// Motor controller for TB6612FNG
pub struct Motor<'d, PWM, IN1, IN2> {
    pwm: PWM,
    in1: Output<'d, IN1>,
    in2: Output<'d, IN2>,
}

impl<'d, PWM, IN1, IN2> Motor<'d, PWM, IN1, IN2> {
    /// Set motor direction and speed
    pub fn set(&mut self, direction: Direction, speed: u8);

    /// Stop motor (coast)
    pub fn stop(&mut self);

    /// Brake motor (short brake)
    pub fn brake(&mut self);
}
```

### PWM設定

| パラメータ | 値 | 備考 |
|-----------|-----|------|
| タイマー | TIM4 | 汎用タイマー |
| 周波数 | 20kHz | 可聴域外（静音動作）|
| 分解能 | 8bit相当 | 0〜255で速度指定 |
| カウントモード | Edge Aligned Up | 標準的なPWM |

### API設計

```rust
/// Create dual motor controller
pub struct DualMotor<'d> {
    motor_a: Motor<'d, ...>,
    motor_b: Motor<'d, ...>,
}

impl DualMotor<'d> {
    /// Move forward (both motors)
    pub fn forward(&mut self, speed: u8);

    /// Move backward (both motors)
    pub fn backward(&mut self, speed: u8);

    /// Turn left (differential drive)
    pub fn turn_left(&mut self, speed: u8);

    /// Turn right (differential drive)
    pub fn turn_right(&mut self, speed: u8);

    /// Stop both motors
    pub fn stop(&mut self);
}
```

## デモ動作シーケンス

1. **起動**: Status LED点灯、初期化完了をdefmtでログ出力
2. **前進**: 50%速度で2秒間
3. **停止**: 1秒間
4. **後退**: 50%速度で2秒間
5. **停止**: 1秒間
6. **左旋回**: 50%速度で1秒間
7. **右旋回**: 50%速度で1秒間
8. **ブレーキ**: 完全停止
9. **ループ**: 手順2に戻る

## 依存クレート

```toml
[dependencies]
embassy-executor = { version = "0.7", features = ["arch-cortex-m", "executor-thread"] }
embassy-stm32 = { version = "0.2", features = ["stm32f411ce", "time-driver-any", "memory-x"] }
embassy-time = { version = "0.4" }
defmt = "0.3"
defmt-rtt = "0.4"
panic-probe = { version = "0.3", features = ["print-defmt"] }
cortex-m = { version = "0.7", features = ["critical-section-single-core"] }
cortex-m-rt = "0.7"
embedded-hal = "1.0"
```

## ファイル構成

```
stm32-embassy/projects/motor-control/
├── .cargo/
│   └── config.toml       # ビルド設定
├── src/
│   └── main.rs           # メインプログラム
├── Cargo.toml            # 依存クレート定義
├── build.rs              # ビルドスクリプト
├── memory.x              # メモリレイアウト
└── rust-toolchain.toml   # Rustツールチェイン
```

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

## 参考資料

- [TB6612FNGデータシート](https://toshiba.semicon-storage.com/jp/semiconductor/product/motor-driver-ics/brushed-dc-motor-driver-ics/detail.TB6612FNG.html)
- [STM32F411リファレンスマニュアル](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411xce-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [Embassy STM32 HAL](https://docs.embassy.dev/embassy-stm32/)
