# Embassy v0.6.0 アップグレード調査メモ

調査日: 2026-06-22

## 対象バージョン

- 現在: `embassy-stm32 = "0.5"` / `embassy-executor = "0.9"`
- 移行先: `embassy-stm32 = "0.6"` / `embassy-executor = "0.10"`

## 破壊的変更（breaking changes）

### gpio-init-analog が非デフォルトに変更

以前はデフォルト有効だった `gpio-init-analog` feature が **デフォルト無効** になった。
未使用ピンをアナログモードで初期化して消費電力を下げる機能で、opt-in が必要になった。

```toml
# 以前の動作を維持したい場合
embassy-stm32 = { version = "0.6", features = [..., "gpio-init-analog"] }
```

**本プロジェクトへの影響**: 動作には影響しない。消費電力がわずかに増える可能性がある程度。

### I2C v2

- `respond_to_write` / `respond_to_read` の戻り値がバッファサイズ → 実転送バイト数に変更

**本プロジェクトへの影響**: I2C 未使用のため無関係。

### FDCAN

- `BusOFF` / `BusPassive` / `BusWarning` をバスエラー enum から削除

**本プロジェクトへの影響**: FDCAN 未使用のため無関係。

### Timer input_capture

- タイマーワードサイズをすべての出力に使用するよう変更

**本プロジェクトへの影響**: `SimplePwm` は対象外のため無関係。

## 主な新機能

| 機能 | 内容 |
|------|------|
| Timer `set_period()` | PWM 周波数をランタイムで変更できる（color-led で有用な可能性あり） |
| GPIO `from_flex()` / `from_input()` | `Flex` / `Input` から変換するコンストラクタ追加 |
| ADC | インジェクション変換・トリガー対応強化 |
| DMA | メモリ間転送対応（BDMA/GPDMA） |
| WWDG | ウィンドウウォッチドッグドライバ追加 |
| embassy-executor 0.10 | タスクメタデータ API、優先度スケジューリング（optional）、EDF スケジューリング（optional）追加 |

## 本プロジェクトへの影響まとめ

| プロジェクト | 影響 | コード修正 |
|------------|------|----------|
| led-blink | なし | 不要 |
| color-led | なし | 不要 |

## アップグレード手順

両プロジェクトの `Cargo.toml` を変更するだけで完了する見込み。

```toml
# 変更前
embassy-executor = { version = "0.9", ... }
embassy-stm32 = { version = "0.5", ... }

# 変更後
embassy-executor = { version = "0.10", ... }
embassy-stm32 = { version = "0.6", ... }
```

`cargo build --release` でコンパイルエラーが出た場合は本ファイルの breaking changes を参照。

## 参考

- 調査ソース: `~/work/external/embassy` （公式リポジトリクローン）
- CHANGELOG: `embassy-stm32/CHANGELOG.md`、`embassy-executor/CHANGELOG.md`
