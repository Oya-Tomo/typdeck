<div align="center">

# typdeck

**Present your Typst slides right in the browser**

<p>
  <img alt="Rust" src="https://img.shields.io/badge/Rust-1.98%2B-ce422b?style=for-the-badge&logo=rust&logoColor=white">
  <img alt="Typst" src="https://img.shields.io/badge/Typst-0.15-239dad?style=for-the-badge&logo=typst&logoColor=white">
  <img alt="Topcoat" src="https://img.shields.io/badge/Topcoat-0.10-1f2937?style=for-the-badge">
</p>

</div>

---

## Features

- 📺 **Viewer** — Shows only the Typst slides. Make the browser full screen and project it
- 🎤 **Presenter console**
  - 👀 The current slide and the next one
  - 📝 Speaker notes alongside, editable in place and saved back to Markdown
  - 🗂️ Thumbnails of all slides, with the current one outlined. Click to jump
  - ⏱️ A timer (start / pause / reset / ±1 min / ±10 s)
- ⚡ **Live reload** — Saving the `.typ` file or the notes updates every view
- 🔗 **Links** — URLs in slides open in a new tab
- 🎬 **GIFs** — Animations play as they are
- 🔁 **Sync** — Every view follows the same slide, even across devices
- 🚨 **Errors** — Typst errors appear in the presenter console while the slides keep their last good state
- 📦 **Packages** — `@preview` packages are downloaded automatically
- 🦀 **Single binary** — Typst is built in, so no `typst` command is needed

## Install

Requires Rust 1.98 or later (required by Topcoat).

```sh
git clone https://github.com/Oya-Tomo/typdeck
cd typdeck
cargo install --path .                  # installs into ~/.cargo/bin
cargo install --path . --root ~/.local  # installs into ~/.local/bin
```

## Usage

```sh
typdeck slides.typ -n notes.md
```

It prints two URLs:

```
viewer:    http://127.0.0.1:3000/viewer
presenter: http://127.0.0.1:3000/presenter
```

### Options

| Option | Description |
|---|---|
| `-n, --notes <FILE>` | Markdown file with speaker notes (created when a note is first edited) |
| `-p, --port <PORT>` | Port to listen on (default: `3000`) |
| `--host <ADDR>` | Address to listen on (default: `127.0.0.1`) |
| `--root <DIR>` | Root for absolute paths in Typst (default: the slides' directory) |

### Keys

| Key | Action |
|---|---|
| `→` `↓` `PageDown` `Space` | Next slide |
| `←` `↑` `PageUp` `Backspace` | Previous slide |
| `Home` / `End` | First / last slide |

## Speaker Notes

Separate the notes with lines containing only `---`. The sections are assigned to the slides in order. `---` inside fenced code blocks is not treated as a separator.

```markdown
# Intro
Say **hello** and introduce yourself
---
Show the link
---
Q&A
```

## Caveats

- 🔓 There is no authentication. With `--host 0.0.0.0`, anyone on the network can move the slides and edit the notes
- 🧪 Topcoat is an experimental framework
- 🗃️ On first launch, the browser runtime is prepared in `~/.cache/typdeck`

---

<div align="center">

Built with [Typst](https://typst.app) and [Topcoat](https://github.com/tokio-rs/topcoat)

</div>
