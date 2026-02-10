# 補助ツール

`tools/` ディレクトリに含まれるユーティリティスクリプト。

## rs2html.py — Rustソースコード → HTML変換

Rustプロジェクトのソースコードをシンタックスハイライト付きHTMLに変換する。
ブラウザで開いて印刷すると、カラーでソースコードを紙に出力できる。

### 前提条件

- Python 3.10+
- Pygments (`pip install pygments` またはシステムパッケージ)

### 基本的な使い方

```bash
cd stm32-embassy

# 単一ディレクトリ
python3 tools/rs2html.py projects/led-blink/

# サブディレクトリも含める（推奨）
python3 tools/rs2html.py -r projects/color-led/
```

出力ファイルはカレントディレクトリに `<ディレクトリ名>_rust.html` として生成される。

### オプション

| オプション | 説明 |
|-----------|------|
| `-r`, `--recursive` | サブディレクトリを再帰的に走査 |
| `-o FILE`, `--output FILE` | 出力ファイル名を指定 |
| `-s STYLE`, `--style STYLE` | Pygmentsカラーテーマを指定（デフォルト: `friendly`） |
| `--list-styles` | 利用可能なテーマ一覧を表示して終了 |

### テーマの選択

```bash
# 利用可能なテーマ一覧（dark/light表示付き）
python3 tools/rs2html.py --list-styles

# ライト系テーマ（印刷向き）
python3 tools/rs2html.py -r -s friendly projects/color-led/
python3 tools/rs2html.py -r -s tango projects/color-led/

# ダーク系テーマ（画面閲覧向き）
python3 tools/rs2html.py -r -s monokai projects/color-led/
```

印刷する場合はライト系テーマ（`friendly`, `default`, `tango` など）を推奨。
ダーク系テーマを選択した場合でも、`@media print` で印刷時のレイアウト調整が適用される。

### 対象ファイル

| パターン | 内容 |
|---------|------|
| `*.rs` | Rustソースコード |
| `Cargo.toml` | プロジェクト設定 |
| `rust-toolchain.toml` | ツールチェイン設定 |
| `.cargo/config.toml` | ビルド設定 |
| `build.rs` | ビルドスクリプト |
| `memory.x`, `*.ld` | リンカスクリプト |

`target/` と `.git/` ディレクトリは自動的に除外される。

### 使用例

```bash
# color-ledプロジェクトをtangoテーマでHTML化
python3 tools/rs2html.py -r -s tango -o color-led.html projects/color-led/

# 出力例:
# Output: color-led.html (4 files, style: tango [light])
#   - .cargo/config.toml
#   - Cargo.toml
#   - rust-toolchain.toml
#   - src/main.rs
```

生成されたHTMLをブラウザで開き、印刷（Ctrl+P）でPDF化または紙に出力できる。

---

## kt2html.py — Kotlinソースコード → HTML変換

Kotlinプロジェクトのソースコードをシンタックスハイライト付きHTMLに変換する。

### 基本的な使い方

```bash
# 単一ディレクトリ
python3 tools/kt2html.py <ディレクトリ>

# サブディレクトリも含める
python3 tools/kt2html.py -r <ディレクトリ>

# 出力ファイル名を指定
python3 tools/kt2html.py -r -o output.html <ディレクトリ>
```

| オプション | 説明 |
|-----------|------|
| `-r`, `--recursive` | サブディレクトリを再帰的に走査 |
| `-o FILE`, `--output FILE` | 出力ファイル名を指定（デフォルト: `<ディレクトリ名>_sources.html`） |
