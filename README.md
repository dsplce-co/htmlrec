> We're dsplce.co, check out our work on our website: [dsplce.co](https://dsplce.co) 🖤

# htmlrec

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![crates.io Size](https://img.shields.io/crates/d/htmlrec?style=for-the-badge&color=%23FF0346)](https://crates.io/crates/htmlrec)
[![crates.io Size](https://img.shields.io/crates/size/htmlrec?style=for-the-badge)](https://crates.io/crates/htmlrec)
[![Licence](https://img.shields.io/crates/l/htmlrec.svg?style=for-the-badge)](https://crates.io/crates/htmlrec)
[![crates.io](https://img.shields.io/crates/v/htmlrec?style=for-the-badge&color=%230F80C1)](https://crates.io/crates/htmlrec)

⚡ Render HTML animations to video — frame-perfect, headless and deterministic.

`htmlrec` (`hrec`) is a command-line tool that captures HTML/CSS/JS animations into video files by driving a virtual clock through a headless Chromium instance, frame by frame.

## 🖤 Features

- `hrec render anim.html -o out.mp4` Got a CSS or JS animation? Capture it to a crisp MP4, no screen recording needed
- `hrec render anim.html -o out.webm --transparent` Need transparency? Export to WebM/VP9 or ProRes 4444 with a proper alpha channel
- Frame-perfect rendering — a virtual clock overrides `performance.now`, `Date.now`, `rAF`, `setTimeout`, `setInterval`, and CSS animations so every frame is deterministic
- `--zoom` support for scaling elements before capture

---

## Table of Contents

- [🖤 Features](#-features)
- [📦 Installation](#-installation)
  - [cargo](#cargo)
  - [npm](#npm)
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

After installation, the `hrec` command will be available in your terminal.

### npm

Install globally from npm — the right prebuilt binary for your platform is fetched automatically:

```bash
npm install -g htmlrec
```

This installs the `hrec` command. Supported platforms: Linux x64, macOS x64, macOS arm64. See [Requirements](#%EF%B8%8F-requirements) for the ffmpeg and Chrome/Chromium prerequisites.

⸻

## 🧪 Usage

### Render an animation to video

Point `hrec` at any HTML file containing a CSS or JS animation and it will render it frame by frame into a video:

```bash
hrec render animation.html
```

By default this produces `output.mp4` at 1280×720, 30 fps, for 5 seconds.

All time-based browser APIs — `performance.now`, `Date.now`, `requestAnimationFrame`, `setTimeout`, `setInterval`, and CSS animations — are overridden by a virtual clock driven by `hrec`. This guarantees that every frame is captured at the exact right moment, regardless of how fast or slow the machine is.

### Render with transparency

Export to WebM (VP9) or MOV (ProRes 4444) to preserve the alpha channel:

```bash
hrec render animation.html -o out.webm --transparent
hrec render animation.html -o out.mov --transparent
```

The `--transparent` flag omits the browser background and captures raw RGBA pixels per frame. Output must use `.webm` or `.mov` — using `--transparent` with `.mp4` is an error.

### Custom resolution and duration

```bash
hrec render animation.html -o out.mp4 --width 1920 --height 1080 --duration 10 --fps 60
```

| Flag | Default | Description |
|---|---|---|
| `-o`, `--output` | `output.mp4` | Output video file |
| `-d`, `--duration` | `5.0` | Duration in seconds |
| `-f`, `--fps` | `30` | Frames per second |
| `--width` | `1280` | Viewport width in pixels |
| `--height` | `720` | Viewport height in pixels |

### Zoom into an element

Scale any element before capture using CSS zoom:

```bash
# Zoom the root element to 2× (default target is :root)
hrec render animation.html -o out.mp4 --zoom 2.0

# Zoom a specific element
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

- **Chromium or Chrome**: A working Chromium/Chrome installation accessible to the system — `htmlrec` launches it headlessly to capture frames
- **ffmpeg**: Used to encode the captured PNG frames into the output video — must be available on your `PATH`
- **cargo**: For installation from source

⸻

## 📁 Repo & Contributions

🛠️ **Repo**: [https://github.com/dsplce-co/htmlrec](https://github.com/dsplce-co/htmlrec)<br/>
📦 **Crate**: [https://crates.io/crates/htmlrec](https://crates.io/crates/htmlrec)

PRs welcome, feel free to contribute

⸻

## 📄 Licence

MIT or Apache-2.0, at your option.
