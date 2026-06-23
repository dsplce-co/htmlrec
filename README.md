> We're dsplce.co, check out our work on our website: [dsplce.co](https://dsplce.co) 🖤

# htmlrec

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![crates.io Size](https://img.shields.io/crates/d/htmlrec?style=for-the-badge&color=%23FF0346)](https://crates.io/crates/htmlrec)
[![crates.io Size](https://img.shields.io/crates/size/htmlrec?style=for-the-badge)](https://crates.io/crates/htmlrec)
[![Licence](https://img.shields.io/crates/l/htmlrec.svg?style=for-the-badge)](https://crates.io/crates/htmlrec)
[![crates.io](https://img.shields.io/crates/v/htmlrec?style=for-the-badge&color=%230F80C1)](https://crates.io/crates/htmlrec)

⚡ Render HTML/CSS animations to video — frame-perfect, headless and deterministic.

`htmlrec` (`hrec`) is a command-line tool that captures HTML/CSS/JS animations into video files by driving a virtual clock through a headless Chromium instance, frame by frame.

## 🖤 Features

- `hrec render anim.html` Got a lovely animation stuck in a browser tab — one Claude one-shotted, or one you hand-rolled — and want it as an actual file? Here's your video, no screen recorder in sight
- `hrec render anim.html -o out.webm --transparent` Need it to drop cleanly over other footage? Export WebM or ProRes with a real alpha channel — the one thing a screen recording can never give you
- Frame-perfect and deterministic — less a camcorder pointed at a monitor, more how a render farm makes a film: a virtual clock walks the page forward a frame at a time, so the same input gives the same bytes out, on your laptop or a throttled CI box alike
- `hrec render anim.html --zoom 2.0 --zoom-element ".logo"` Just want to frame one detail? Scale any element before the capture

---

## Table of Contents

- [🖤 Features](#-features)
- [📦 Installation](#-installation)
  - [cargo](#cargo)
  - [npm](#npm)
  - [Homebrew](#homebrew)
  - [`.deb` file](#deb-file)
- [💪 Agent skill](#-agent-skill)
- [🧪 Usage](#-usage)
  - [Render an animation to video](#render-an-animation-to-video)
  - [Render with transparency](#render-with-transparency)
  - [Custom resolution and duration](#custom-resolution-and-duration)
  - [Zoom into an element](#zoom-into-an-element)
  - [Supported output formats](#supported-output-formats)
- [🛠️ Requirements](#%EF%B8%8F-requirements)
- [📁 Repo & Contributions](#-repo--contributions)
- [📄 Licence](#-licence)

⸻

## 📦 Installation

### cargo

Install from crates.io using cargo:

```bash
cargo install htmlrec
```

Alternatively you can find pre-built binaries for your platform on [GitHub](https://github.com/dsplce-co/htmlrec/releases).

After installation, the `hrec` command will be available in your terminal.

`hrec` also needs **ffmpeg** on your `PATH` to encode the video — grab it from your package manager if you haven't already.

### npm

Install globally from npm — the right prebuilt binary for your platform is fetched automatically:

```bash
npm install -g htmlrec
```

This installs the `hrec` command. Supported platforms: Linux x64, macOS x64, macOS arm64. See [Requirements](#%EF%B8%8F-requirements) for the ffmpeg and Chrome/Chromium prerequisites.

### Homebrew

Install from our tap:

```bash
brew install dsplce-co/tap/htmlrec ffmpeg
```

This makes the `hrec` command available in your terminal.

### `.deb` file

You can find the latest `.deb` file on [GitHub Releases](https://github.com/dsplce-co/htmlrec/releases/latest).

⸻

## 💪 Agent skill

Working with a coding agent (Claude Code, Cursor, and friends)? We ship an agent skill so it can drive `hrec` for you — hand it the animation, get the video back. Add it with the [`skills`](https://github.com/vercel-labs/skills) CLI:

```bash
npx skills add dsplce-co/htmlrec
```

⸻

## 🧪 Usage

### Render an animation to video

You've got an animation built in plain HTML/CSS/JS — a loader, a logo reveal, some little canvas thing — and you want it as an actual video file. The obvious move is to screen-record it, which works right up until you look closely: a frame drops here, it runs a touch fast there, and re-running it never hands you the same file twice. So point `hrec` at the HTML and let it capture the thing properly instead:

```bash
hrec render animation.html
```

By default this writes `output.webm` at 1280×720, 60 fps, for 5 seconds. Want a standard, plays-anywhere MP4? Just name it:

```bash
hrec render animation.html -o out.mp4
```

Under the hood every time-based browser API — `performance.now`, `Date.now`, `requestAnimationFrame`, `setTimeout`, `setInterval`, and CSS animations — is overridden by a virtual clock that `hrec` steps forward one frame at a time. The page believes a steady ~16ms passed between frames even whilst your machine took two seconds to draw one, so every frame lands at exactly the right moment, no matter how fast or slow the box.

### Render with transparency

Sometimes you want the animation to sit *over* something else — a video, a slide, a background — with everything around it cut out. Render to WebM (VP9) or MOV (ProRes 4444) and pass `--transparent` to keep the alpha channel:

```bash
hrec render animation.html -o out.webm --transparent
hrec render animation.html -o out.mov --transparent
```

`--transparent` drops the browser background and captures raw RGBA per frame. It needs a `.webm` or `.mov` output — ask for it with `.mp4` and `hrec` will stop you, since MP4 simply can't carry alpha.

### Custom resolution and duration

```bash
hrec render animation.html -o out.mp4 --width 1920 --height 1080 --duration 10 --fps 60
```

| Flag | Default | Description |
|---|---|---|
| `-o`, `--output` | `output.webm` | Output file — the extension picks the format (see below) |
| `-d`, `--duration` | `5.0` | Duration in seconds |
| `-f`, `--fps` | `60` | Frames per second |
| `--width` | `1280` | Viewport width in pixels |
| `--height` | `720` | Viewport height in pixels |
| `-t`, `--transparent` | off | Keep the alpha channel — needs `.webm` or `.mov` |
| `--zoom` | — | CSS zoom factor applied before capture (e.g. `2.0`, `0.5`) |
| `--zoom-element` | `:root` | CSS selector the `--zoom` applies to |

### Zoom into an element

Scale any element before capture using CSS zoom — handy for framing a detail or bumping up something small:

```bash
# Zoom the whole page to 2× (default target is :root)
hrec render animation.html -o out.mp4 --zoom 2.0

# Zoom one specific element
hrec render animation.html -o out.mp4 --zoom 0.5 --zoom-element ".card"
```

### Supported output formats

| Extension | Codec | Alpha |
|---|---|---|
| `.mp4` | H.264 / yuv420p | No |
| `.webm` | VP9 / yuva420p | Yes |
| `.mov` | ProRes 4444 / yuva444p10le | Yes |

**Example HTML file:**

`animation.html`:

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body { margin: 0; background: #000; }
    .box {
      width: 100px; height: 100px;
      background: #fff;
      animation: slide 2s linear infinite;
    }
    @keyframes slide {
      from { transform: translateX(0); }
      to   { transform: translateX(500px); }
    }
  </style>
</head>
<body>
  <div class="box"></div>
</body>
</html>
```

```bash
hrec render animation.html -o out.mp4 --duration 2
```

⸻

## 🛠️ Requirements

- **Chromium or Chrome**: `hrec` uses any Chrome/Chromium already on your system, and otherwise downloads a headless Chromium on first run, caching it for next time. On Linux the downloaded browser may still want a few common system libraries a desktop browser usually brings along — if any are missing, `hrec` passes through the browser's own error naming them
- **ffmpeg**: required, on your `PATH` — it encodes the captured frames into the final video

⸻

## 📁 Repo & Contributions

🛠️ **Repo**: [https://github.com/dsplce-co/htmlrec](https://github.com/dsplce-co/htmlrec)<br>
📦 **Crate**: [https://crates.io/crates/htmlrec](https://crates.io/crates/htmlrec)

PRs welcome, feel free to contribute

⸻

## 📄 Licence

MIT or Apache-2.0, at your option.
