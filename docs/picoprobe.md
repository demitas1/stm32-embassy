# Picoprobe → STM32 BlackPill 配線ガイド

## ピン対応表

| Pico GPIO | Pico 物理ピン | BlackPill | 説明 |
|-----------|-------------|-----------|------|
| GP2 | Pin 4 | PA14 | SWCLK |
| GP3 | Pin 5 | PA13 | SWDIO |
| GP15 | Pin 20 | NRST | リセット（任意） |
| GND | Pin 3 など | GND | グランド |
| 3V3(OUT) | Pin 36 | 3.3V | 給電（任意） |
| GP4 | Pin 6 | PA10 (RX) | UART TX → BlackPill RX（任意） |
| GP5 | Pin 7 | PA9 (TX) | UART RX ← BlackPill TX（任意） |

## Pico ピン配置（該当箇所抜粋）

```
                    USB
              ┌─────────────┐
    GP0  (1) ─┤             ├─ (40) VBUS
    GP1  (2) ─┤             ├─ (39) VSYS
    GND  (3) ─┤  ★GND      ├─ (38) GND
    GP2  (4) ─┤  ★SWCLK    ├─ (37) 3V3_EN
    GP3  (5) ─┤  ★SWDIO    ├─ (36) 3V3(OUT) ★給電
    GP4  (6) ─┤  ★UART TX  ├─ (35) ADC_VREF
    GP5  (7) ─┤  ★UART RX  ├─ (34) GP28
         ...                ...
    GP15(20) ─┤  ★NRST     ├─ ...
              └─────────────┘
```

## 最小構成（3本）

デバッグ・書き込みだけなら最低これだけで動作します。

```
Pico GP2  ──── BlackPill PA14 (SWCLK)
Pico GP3  ──── BlackPill PA13 (SWDIO)
Pico GND  ──── BlackPill GND
```

> BlackPill を USB で給電する場合は電源線不要です。

## 推奨構成（UART込み）

```
Pico GP2  ──── BlackPill PA14 (SWCLK)
Pico GP3  ──── BlackPill PA13 (SWDIO)
Pico GP4  ──── BlackPill PA10 (UART1 RX)
Pico GP5  ──── BlackPill PA9  (UART1 TX)
Pico GP15 ──── BlackPill NRST
Pico GND  ──── BlackPill GND
```

UART を接続しておくと、picoprobe が SWD プログラマ＋シリアルモニタを同時に 1 本の USB で提供してくれるので非常に便利です。

## 注意点

- BlackPill と Pico をそれぞれ別の USB で給電する場合は GND のみ共通にすれば十分（3.3V 線は不要）
- NRST はなくても書き込み・デバッグは基本的に動作します
- BlackPill の PA13/PA14 は SWD 専用ピンなのでそのまま使えます
