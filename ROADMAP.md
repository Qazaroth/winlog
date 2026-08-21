# Roadmap & Development Strategy 🗺️

This document outlines the development phases, planned features, and architectural milestones for **`winlog`**. 

Since `winlog` is designed to be an accessible, high-performance open-source project, milestones are grouped into logical phases so contributors can pick up tasks at any stage.

---

## 🎯 Project Goals

1. **Speed & Efficiency:** Process and render tens of thousands of Windows events per second with zero UI freezing.
2. **Developer-Friendly Ergonomics:** Provide clean CLI piping (`--json`, `--ndjson`), human-readable descriptions for cryptic codes, and intuitive vim-like keybindings in the TUI.
3. **Low Barrier to Contribution:** Keep the core engine decoupled from TUI/CLI rendering and preset definitions.

---

## 🚩 Phase 1: Core Engine & Native Bindings (MVP)

Focus on building a fast, low-level wrapper around the native Windows Event Log API (`winevt.dll`).

- [x] **Windows API Bindings:**
  - [x] Implement wrapper around `EvtQuery` and `EvtNext` using `windows-rs`.
  - [x] Support reading active live channels (`System`, `Security`, `Application`).
  - [x] Support parsing offline static `.evtx` files.
- [x] **Data Model & Schema:**
  - [x] Define structured `EventRecord` struct (Timestamp, Provider, Event ID, Level, Computer Name, Payload parameters).
  - [x] Implement robust XML-to-struct parser for event payloads.
- [x] **Basic CLI Interface:**
  - [x] Implement arguments via `clap` (channel selection, limit, output format).
  - [x] Output formatted terminal logs with basic color coding based on severity level (Error, Warning, Information).

---

## 🚩 Phase 2: Live Tailing & Export Capabilities

Focus on real-time log monitoring and integration with terminal pipelines.

- [x] **Live Tail Engine:**
  - [x] Implement asynchronous streaming subscriber using `EvtSubscribe`.
  - [x] Support live auto-scrolling terminal output (`winlog tail -c System`).
- [x] **Structured Exports:**
  - [x] Add JSON (`--json`) and NDJSON (`--ndjson`) output modes.
  - [x] Support direct stdout piping (`winlog --json | jq '.'`).
  - [x] Export filtered queries to `.csv` and `.json` files.

---

## 🚩 Phase 3: Interactive TUI (Terminal User Interface)

Focus on building a responsive, feature-rich TUI that directly competes with `eventvwr.msc`.

- [x] **Split-Pane UI Architecture:**
  - [x] Virtualized, high-performance scrollable Event Table (Timestamp, Level, ID, Source, Message summary).
  - [x] Collapsible Detail Pane showing formatted key-value event parameters and raw XML view.
  - [x] Status bar showing current channel, active filter, total event count, and resource usage.
- [x] **Keybindings & Ergonomics:**
  - [x] Vim-style navigation (`j`, `k`, `g`, `G`, `/`).
  - [x] Quick log level toggles (`1` for Error, `2` for Warning, `3` for Info).
  - [x] Copy formatted event summary or raw XML to system clipboard.
- [x] **Interactive Search:**
  - [x] Sub-millisecond fuzzy search across rendered events (`fzf`-style).
  - [x] Regex query input box with live syntax checking.

---

## 🚩 Phase 4: Intelligence & Community Presets

Focus on making raw Windows logs actionable and context-rich.

- [x] **Context & Lookup Engine:**
  - [x] Embed human-readable description dictionary for top 100+ Windows Event IDs (e.g., Event 4624 $
ightarrow$ "Successful Logon").
  - [x] Automatically translate common HRESULT / NTSTATUS hex error codes (`0x80070005` $
ightarrow$ "Access Denied").
  - [x] Provide quick lookup links to Microsoft Documentation / Sysmon references.
- [ ] **Rule & Filter Presets:**
  - [x] Support custom YAML/TOML preset configurations.
  - [ ] Ship with built-in audit presets:
    - `security-audit`: Highlights failed logons, account lockouts, privilege escalations.
    - `system-errors`: Filters driver crashes, service failures, disk warnings.
    - `network-activity`: Aggregates firewall drops and connection events.

---

## 🚩 Phase 5: Plugin Architecture & Advanced Features

- [ ] **Remote Event Log Querying:** Query remote Windows machines via WinRM / WMI protocols.
- [ ] **Custom Event Provider Schema Extensions:** Support dynamic schema loading for third-party application logs.
- [ ] **Plugin System:** Allow external Rust crates or WASM modules to define custom event processors and alerting outputs.

---

## 🤝 How to Pick Up a Task

If you're looking to contribute:
1. Check the [Issue Tracker](https://github.com/Qazaroth/winlog/issues) for tasks labeled `good first issue` or `help wanted`.
2. Pick any unassigned checkbox from Phase 1 or Phase 2.
3. Open a Draft PR early so others know you're working on it!
