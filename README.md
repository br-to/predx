# predx

Polymarket と Kalshi を横断検索して確率を並べて表示する CLI ツール。

## インストール

```bash
cargo install --path .
```

## 使い方

```bash
# 基本の検索
predx search "trump"

# 表示件数を指定（デフォルト: 20、最大: 100）
predx search "bitcoin" -l 5
```

### 出力例

```
Polymarket (5/44)                       │  Kalshi (5/774)
──────────────────────────────────────  │  ──────────────────────────────────────
Will the price of B... 100.0%    159.1k  │  When will Bitcoin h...  12.0%    121.3k
Will the price of B... 100.0%    102.6k  │  When will Bitcoin h...   5.0%     83.0k
Will the price of B... 100.0%    187.3k  │  When will Bitcoin h...   5.0%    183.6k
Will the price of B... 100.0%    247.9k  │  When will Bitcoin h...   2.0%     27.4k
Will the price of B... 100.0%    405.1k  │  When will Bitcoin h...   1.0%  11014.0k
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
