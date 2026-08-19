 # winlog

> A fast, modern CLI and TUI alternative to the Windows Event Viewer (`eventvwr.msc`). Built in Rust for high performance, real-time log streaming, and human-readable context.

---

## Why `winlog`?

The default Windows Event Viewer (`eventvwr.msc`) has been a source of frustration for sysadmins, security analysts, and developers for years:
- **Slow & Heavy:** Freezes on large `.evtx` files and uses single-threaded UI rendering.
- **Vague & Cryptic:** Buries critical parameters inside deep XML tabs and leaves raw HRESULT codes (`0x80070005`) uninterpreted.
- **Poor Searchability:** Requires complex XPath / XML query builders to perform basic filtering.

`winlog` solves this by bringing a fast, intuitive terminal workflow to Windows Event Log management.

---

## ✨ Features (In Progress & Planned)

- [ ] **🚀 Blazing Fast TUI:** Responsive split-pane terminal user interface powered by [`ratatui`](https://github.com/ratatui-org/ratatui).
- [ ] **⚡ Sub-Millisecond Search:** Instant fuzzy filtering (`fzf`-style) and regex search across log channels.
- [ ] **📡 Live Tail Streaming:** Real-time log monitoring (`tail -f` equivalent) for active Windows channels (`System`, `Security`, `Application`).
- [ ] **🧠 Human-Readable Context:** Automatic translation of obscure Event IDs and hex error codes into plain-English explanations.
- [ ] **📤 Structured Export:** Clean JSON/NDJSON output for seamless integration with terminal pipelines (`| jq`).
- [ ] **🎯 Filter Presets:** Pre-packaged YAML/TOML presets for quick security auditing and troubleshooting.

---

## 🛠️ Requirements

- **OS:** Windows 10 / 11 / Windows Server 2016+
- **Toolchain:** [Rust](https://www.rust-lang.org/tools/install) (1.70+)

---

## 🚀 Quick Start

### Building from Source

```bash
# Clone the repository
git clone https://github.com/Qazaroth/winlog.git
cd winlog

# Build and run using Cargo
cargo run --release
```

---

## 🤝 Contributing

Contributions are very welcome! Whether you are a seasoned Rust developer or just starting out, there are plenty of ways to contribute:
1. **Adding Event ID explanations:** Help expand the human-readable dictionary for common Windows Event IDs.
2. **Improving TUI UX:** Propose or build UI improvements in `ratatui`.
3. **Bug Fixes & Feature Requests:** Open an issue or submit a PR.

Please check out `CONTRIBUTING.md` *(coming soon)* before submitting pull requests.

---

## 📄 License

This project is licensed under the **GNU General Public License v3.0** (`GPL-3.0`). See the [LICENSE](LICENSE) file for details.

---

## 🤖 Note on Development & AI Assistance

This project is created as a long-term learning journey in **Rust** and low-level Windows systems programming. Because I am currently learning Rust, AI assistance is utilized during development to help design architecture, navigate complex Windows APIs, and write idiomatically structured Rust code. 

Contributions and code reviews from experienced Rust developers are especially welcome!
