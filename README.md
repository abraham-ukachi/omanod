<p align="center">
  <b>OmaNod</b> is an Omarchy-native personal agent, written in <b>Rust</b> 🦀. It is a fork of <a href="https://github.com/n8n-io/nod">n8n-io/nod</a>, rebuilt from the ground up for Linux desktops that already have a theme, a terminal, and no patience for Electron.
</p>

<p align="center">
  <a href="https://github.com/abraham-ukachi/omanod/releases"><b>Releases &rarr;</b></a>
</p>

# `omanod`

> IMPORTANT: This is a work in progress. The first milestone is **chat + models + themes**. Nod's browser, sandbox, scheduling and channel integrations are on the roadmap, not in this build yet.


## What is OmaNod? 🤖

Picture your desktop 🖥️: the terminal has a theme, the browser has a theme, even the music player has a theme — and then you open an AI app and it looks like it was designed in a different universe 😅.

**OmaNod** is the personal agent that looks like it shipped with [Omarchy](https://omarchy.org). It lives in a dense three-zone window — conversations on the left, messages in the middle, composer at the bottom — and speaks over the model provider of your choice. It wears every **Omarchy theme** 🎨 by reading `colors.toml` live: background, foreground, accent, and a few fixed mixes. iA Writer Mono S, bundled under the SIL OFL 1.1, keeps the text calm.

The big idea from [nod](https://nod.n8n.io) stays: **your machine, your model, your word**. The first release keeps the chat surface and the model catalog; the trust, browser and computer-use layers come next.

> MOTTO: Native first, features second 😜


## Status 🧪

| Milestone | State |
|:----------|:------|
| 1 · Chat + models + themes | ✅ in progress |
| 2 · Tools, skills, local files | ⏳ next |
| 3 · Trust, approvals, vault | ⏳ next |
| 4 · Browser + computer use | ⏳ later |
| 5 · Channels, schedules, notifications | ⏳ later |


## What works today ✨

| No. | Feature | What it does |
|:----|:--------|:-------------|
| 1 | *`Chat`* | New conversation, select, delete, and persisted history in SQLite |
| 2 | *`Streaming`* | Token-by-token replies with a live caret and token usage in the status bar |
| 3 | *`Providers`* | One catalog covering nod's 38 providers plus local Ollama and xAI Grok, with key status from the environment |
| 4 | *`Models`* | Fetches `/models` on demand, remembers the selected provider and model |
| 5 | *`Themes`* | Live Omarchy `colors.toml` plus 23 built-in themes, switched in Settings |
| 6 | *`Shortcuts`* | `Enter` sends, `Shift+Enter` is a newline, `Ctrl+1/2/3` switches views |


## Model providers 🔌

Keys are read from environment variables — they are never written into config or stored in SQLite. Local Ollama needs no key. OpenAI-compatible, Anthropic and Gemini endpoints stream today; providers that need vendor-specific auth or routing are listed with a *not wired* state until their client lands.

| Kind | Providers |
|:-----|:----------|
| OpenAI-compatible | Ollama (local), xAI Grok, OpenAI, OpenRouter, Groq, DeepSeek, Mistral, Together, Fireworks, Cerebras, Baseten, Moonshot, MiniMax, NVIDIA, Hugging Face, Z.ai, Vercel AI Gateway, generic compatible endpoint |
| Anthropic | Anthropic (`ANTHROPIC_API_KEY`) |
| Gemini | Google Gemini (`GEMINI_API_KEY`) |
| Not wired yet | Azure, Bedrock, Vertex, Cloudflare, OpenCode, Qwen, Xiaomi, Kimi, Meta, Radius, Ant Ling |


## Tech stack & versions 🧰

Versions as built on 2026-11-22:

| Tool | Version | Notes |
|:-----|:--------|:------|
| [Rust](https://www.rust-lang.org) | **stable** (1.99) | Edition 2021. Release profile: `opt-level = "s"`, LTO, stripped. |
| [Slint](https://slint.dev) | **1.19** | The UI lives in [`app/ui/`](./app/ui). |
| [ureq](https://github.com/algesten/ureq) | **2.12** | Blocking HTTP/SSE streaming clients, run off the UI thread. |
| [rusqlite](https://github.com/rusqlite/rusqlite) | **0.32** | Threads, messages and settings in one local database. |
| [iA Writer Mono](https://github.com/iaolo/iA-Fonts) | S | The font of Omacalc, bundled under the SIL OFL 1.1. |


## How it works 🦀⚡

| No. | Piece | File | What it does |
|:----|:------|:-----|:-------------|
| 1 | *`Themes`* | [crates/omanod-core/src/theme.rs](./crates/omanod-core/src/theme.rs) | Reads Omarchy's three colors and derives every surface, border and muted tone from fixed mixes |
| 2 | *`Models`* | [crates/omanod-core/src/models.rs](./crates/omanod-core/src/models.rs) | The provider catalog: endpoint, key variable, protocol kind and an honest status |
| 3 | *`Clients`* | [crates/omanod-core/src/client.rs](./crates/omanod-core/src/client.rs) | OpenAI-compatible, Anthropic and Gemini SSE streaming behind one `stream()` function |
| 4 | *`Store`* | [crates/omanod-core/src/store.rs](./crates/omanod-core/src/store.rs) | SQLite threads, messages and settings under `$XDG_DATA_HOME/omanod` |
| 5 | *`Config`* | [crates/omanod-core/src/config.rs](./crates/omanod-core/src/config.rs) | Theme, provider, model and compatible endpoint in `~/.config/omanod/config.toml` |
| 6 | *`UI`* | [app/ui/app.slint](./app/ui/app.slint) | Sidebar, conversation, models and settings — in Slint |
| 7 | *`App`* | [app/src/main.rs](./app/src/main.rs) | Wires the UI to the core; a small event loop carries streamed tokens back to the message list |


## Install on Omarchy 📦

OmaNod is a native binary: no Electron, no Node runtime, no server. Pick one of the two installs below. Both add a launcher entry, so **SUPER + MENU → OmaNod** works right away.

**One-line user install** (no sudo, binary in `~/.local/bin`):

```sh
curl -fsSL https://raw.githubusercontent.com/abraham-ukachi/omanod/main/scripts/install.sh | bash
```

Or from a clone:

```sh
git clone https://github.com/abraham-ukachi/omanod.git
cd omanod
./scripts/install.sh
```

**Arch package** (system-wide, managed by pacman):

```sh
git clone https://github.com/abraham-ukachi/omanod.git
cd omanod/packaging
makepkg -si
```

Then open the Omarchy menu with **SUPER + MENU** (or **SUPER + SPACE** on the default keymap), type `OmaNod`, and hit Enter. The app follows your live Omarchy theme on first launch. To remove the user install, delete `~/.local/bin/omanod`, `~/.local/share/applications/omanod.desktop`, and the `omanod` icons under `~/.local/share/icons/hicolor`.

> Full details, including AUR-style usage and troubleshooting, live in [Install on Omarchy](./docs/src/content/docs/get-started/install-omarchy.mdx).


## Build & run 🛠️

```sh
cargo build --release           # build the native binary
cargo run -p omanod             # open the window on your desktop
```

Set a key for the provider you want, or start Ollama locally:

```sh
export XAI_API_KEY='...'        # xAI Grok
export OPENAI_API_KEY='...'     # OpenAI
export ANTHROPIC_API_KEY='...'  # Anthropic
export GEMINI_API_KEY='...'     # Google Gemini
export OLLAMA_API_KEY=''        # not needed; Ollama is the default provider
```

Then open **MODELS** › **REFRESH MODELS**, pick a model, and chat. The selected provider and model are saved for next launch.


## Themes 🎨

OmaNod follows `~/.local/state/omarchy/current/theme/colors.toml` — change your Omarchy theme and the app repaints within a second. Pick **Omarchy** in Settings for live theming, or choose one of the 23 built-in themes (Tokyo Night, Gruvbox, Rose Pine, Flexoki Light, and friends). Every color is either one of the three theme colors or a fixed mix of background into foreground, exactly like [Omacalc](https://github.com/omacom-io/omacalc).


## Attribution & license 📄

This repository is a fork of [n8n-io/nod](https://github.com/n8n-io/nod), copyright 2026 n8n GmbH, licensed under **Apache-2.0**. The upstream [`LICENSE`](./LICENSE) and [`NOTICE`](./NOTICE) are kept intact and must stay with any redistribution.

New Rust code written for OmaNod is **MIT**, © 2026 Abraham Ukachi, as declared in the crate manifests. iA Writer Mono is © 2018 Information Architects Inc., licensed under the SIL Open Font License 1.1 ([`OFL.txt`](./app/ui/fonts/OFL.txt)).


## Roadmap 🚀

1. Stop / interrupt, retries off by default, and a single turn lock per conversation.
2. Tools and skills behind one trust check, with approvals bound to exact arguments.
3. Local files and computer use on Linux/Omarchy only.
4. Browser automation and the sandbox that keeps provider keys out of it.
5. Channels, schedules and notifications — each owner-scoped, each redacted before it leaves the machine.

> The upstream architecture, trust model and docs remain the design reference: [`docs/`](./docs/src/content/docs/how-it-works/architecture.mdx).
