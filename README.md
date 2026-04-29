# predx

Polymarket と Kalshi を横断検索して確率を並べて表示する CLI ツール。

## インストール

```bash
cargo install --path .
```

## 使い方

```bash
# 基本の検索（アクティブな市場のみ）
predx search "trump"

# 表示件数を指定（デフォルト: 20、最大: 100）
predx search "bitcoin" -l 5

# 確率順でソート（デフォルトは出来高順）
predx search "trump" --sort prob

# 解決済み市場も含める
predx search "trump" --inactive

# JSON/CSV で出力（他ツール連携・パイプ処理向け）
predx search "trump" --format json
predx search "trump" --format csv | column -t -s,

# 確率変動を定期的に監視（しきい値を超えたら通知）
predx watch "trump 2028" --interval 60 --threshold 5 --limit 5
# --duration 30 で30分後に自動終了。Ctrl+Cで即停止

# Discord / Slack webhook に変動アラートを飛ばす
predx watch "bitcoin" --threshold 3 --webhook https://discord.com/api/webhooks/...
predx watch "bitcoin" --threshold 3 --webhook https://hooks.slack.com/services/...

# ファイルに追記記録（baseline とアラートをすべて残す）
predx watch "trump 2028" --log predx.log

# 特定マーケットをID指定で追跡（複数指定可・プラットフォーム混在可）
predx watch -m polymarket:1733817 -m kalshi:KXPRESNOMD-28-GN
```

### 出力例

```
Polymarket (5/16)                                                                │  Kalshi (5/375)
───────────────────────────────────────────────────────────────────────────────  │  ──────────────────────────────────────────────────────────────────────
Trump announces end of military operations against Iran by April 30th?  36.5%   5741.4k  │  2028 Democratic presidential nominee — Gavin Newsom    25.0%  11146.8k
Will Trump post "Make Iran Great Again" on Truth Social this week?       0.1%   4349.7k  │  2028 Democratic presidential nominee — Mark Kelly       4.0%   8232.9k
Trump announces end of military operations against Iran by June 30th?  81.0%   2132.9k  │  Donald Trump out as President? — Before 2027          16.0%   8076.2k
Trump announces end of military operations against Iran by April 21st? 14.0%   2079.5k  │  2028 Democratic presidential nominee — AOC              9.0%   8041.0k
Trump announces US x Iran ceasefire end by April 18, 2026?              2.0%   1337.8k  │  Will marijuana be rescheduled by July 1?               1.0%   9499.0k
```

## 開発

```bash
# ビルド
cargo build

# テスト
cargo test

# 開発中の実行
cargo run -- search "trump"
```
