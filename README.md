<div align="center">

# typdeck

**Typst スライドを、ブラウザでそのままプレゼン**

<p>
  <img alt="Rust" src="https://img.shields.io/badge/Rust-1.98%2B-ce422b?style=for-the-badge&logo=rust&logoColor=white">
  <img alt="Typst" src="https://img.shields.io/badge/Typst-0.15-239dad?style=for-the-badge&logo=typst&logoColor=white">
  <img alt="Topcoat" src="https://img.shields.io/badge/Topcoat-0.10-1f2937?style=for-the-badge">
</p>

</div>

---

## Features

- 📺 **ビューア** — Typst のスライドだけを表示。ブラウザを全画面にして投影
- 🎤 **プレゼンター画面**
  - 👀 現在のスライドと次のスライド
  - 📝 スピーカーノートを横に表示。その場で編集して Markdown に保存
  - 🗂️ 全スライドのサムネイル一覧。現在のスライドを枠でハイライトし、クリックでジャンプ
  - ⏱️ タイマー（開始 / 一時停止 / リセット / ±1分・±10秒）
- ⚡ **ライブリロード** — `.typ` やノートを保存すると自動で画面を更新
- 🔗 **リンク** — スライド内の URL は新しいタブで開く
- 🎬 **GIF** — アニメーションをそのまま再生
- 🔁 **同期** — どの画面で操作しても、ビューアとプレゼンターが一緒に動く（別端末でも可）
- 🚨 **エラー表示** — Typst のエラーをプレゼンター画面に表示し、スライドは直前の成功状態を維持
- 📦 **パッケージ** — `@preview` パッケージを自動でダウンロード
- 🦀 **単体バイナリ** — Typst を組み込んでいるので `typst` コマンドは不要

## Install

Rust 1.98 以上が必要です（Topcoat の要件）。

```sh
git clone https://github.com/Oya-Tomo/typdeck
cd typdeck
cargo install --path .                  # ~/.cargo/bin にインストール
cargo install --path . --root ~/.local  # ~/.local/bin にインストール
```

## Usage

```sh
typdeck slides.typ -n notes.md
```

起動すると 2 つの URL が表示されます。

```
viewer:    http://127.0.0.1:3000/viewer
presenter: http://127.0.0.1:3000/presenter
```

### Options

| オプション | 説明 |
|---|---|
| `-n, --notes <FILE>` | ノートの Markdown ファイル（無ければ編集時に作成） |
| `-p, --port <PORT>` | ポート番号（既定: `3000`） |
| `--host <ADDR>` | 待ち受けアドレス（既定: `127.0.0.1`） |
| `--root <DIR>` | Typst の絶対パスの基準（既定: スライドのディレクトリ） |

### Keys

| キー | 動作 |
|---|---|
| `→` `↓` `PageDown` `Space` | 次へ |
| `←` `↑` `PageUp` `Backspace` | 前へ |
| `Home` / `End` | 最初 / 最後 |

## Speaker Notes

`---` だけの行で区切ると、上から順に 1 枚目、2 枚目…のスライドに割り当てられます。コードブロック内の `---` は区切りとして扱いません。

```markdown
# はじめに
**挨拶**して自己紹介
---
ここでリンクを紹介
---
質疑応答
```

## Caveats

- 🔓 認証はありません。`--host 0.0.0.0` で公開すると、同じネットワークの誰でもスライド操作とノート編集ができます
- 🧪 Topcoat は実験段階のフレームワークです
- 🗃️ 初回起動時に、ブラウザ用の JS を `~/.cache/typdeck` に用意します

---

<div align="center">

Built with [Typst](https://typst.app) and [Topcoat](https://github.com/tokio-rs/topcoat)

</div>
