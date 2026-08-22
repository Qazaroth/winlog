## [unreleased]

### 📚 Documentation

- Updated ROADMAP for a better phase 5 development, previous ones were out of scope
([`8ade77e…`](https://github.com/Qazaroth/feckmania/commit/8ade77e60d74fb2644b4bf5534d3cdfaff8f3fb8))


### ⚙️ Continuous Integration

- Updated release.yml to hopefully generate executables
([`cc92489…`](https://github.com/Qazaroth/feckmania/commit/cc924891d1827292f6e2bbb4db04eb4809556935))

## [0.6.0] - 2026-08-21

### 🚀 Features

- Human-readable description implemented, more can be added later on
([`94f1421…`](https://github.com/Qazaroth/feckmania/commit/94f14218b3a965e0d5790f1ae725a4e56e3cf276))

- Common hex error codes translated to be human readable
([`276c7f6…`](https://github.com/Qazaroth/feckmania/commit/276c7f61d821f711fe86c521aa652ff385ca216f))

- Quick lookup links to Microsoft Docs or Sysmon ref implemented
([`a5fb27b…`](https://github.com/Qazaroth/feckmania/commit/a5fb27be51fb15f58b99cff6116032a57b03d02c))

- Custom YAML preset config support implemented
([`26a8cbd…`](https://github.com/Qazaroth/feckmania/commit/26a8cbdc899dedb6b001068ffc3f55c161e9768e))

- User is able to specify what preset to use
([`344cf87…`](https://github.com/Qazaroth/feckmania/commit/344cf8716d05bf7141ebc03b8a601280bd8ce075))

- Built-in audit presets implemented
([`d72b019…`](https://github.com/Qazaroth/feckmania/commit/d72b019e41bafcd673a82ca7f9761e4b33dfb347))


### 📚 Documentation

- Updated README to reflect what features are implemented
([`e7ef43c…`](https://github.com/Qazaroth/feckmania/commit/e7ef43ca7b38317c3196290e893ac311315b111b))

## [0.5.0] - 2026-08-21

### 🚀 Features

- Scrollable event table tui implemented
([`00aec90…`](https://github.com/Qazaroth/feckmania/commit/00aec9023e6a90abef90d183ab48d3a012d662ea))

- Collapsible detail pane implemented.
([`4ad3c28…`](https://github.com/Qazaroth/feckmania/commit/4ad3c28db47f66b39f63d7509802433eb9a13fc0))

- Status bar showing some data implemented
([`8cc8cb7…`](https://github.com/Qazaroth/feckmania/commit/8cc8cb788c158675e75c2e593063b741f6957901))

- Vim-style navigation implemented. :)
([`90f540a…`](https://github.com/Qazaroth/feckmania/commit/90f540acbb6bd7d178017efde3efe7de0ce39f36))

- Log-level toggles implemented
([`2db7c08…`](https://github.com/Qazaroth/feckmania/commit/2db7c088ef78451566878d277d538367865d4e89))

- Able to now copy formatted event summary or raw XML
([`4cb4a87…`](https://github.com/Qazaroth/feckmania/commit/4cb4a87cde7c2e8d2d659421895091466a7fdb53))

- Fuzzy search implemented
([`4ae0478…`](https://github.com/Qazaroth/feckmania/commit/4ae0478f368f79d566d49483b46ea2bfe2899c6a))

- Regex search implemented.
([`7f1b09b…`](https://github.com/Qazaroth/feckmania/commit/7f1b09b2778ef2337d4792a247434479ad65b247))

## [0.4.0] - 2026-08-21

### 🚀 Features

- Asynchronous streaming subscriber implemented using 'EvtSubscribe'
([`6372fee…`](https://github.com/Qazaroth/feckmania/commit/6372fee281324abd679091684b5ec3b9e457bccd))

- Live auto-scrolling terminal output implemented.
([`985b6e8…`](https://github.com/Qazaroth/feckmania/commit/985b6e8ab42d60b2c22ebe89e3830b7eebf85194))

- JSON and NDJSON output implemented along with stdout piping
([`37cf2c2…`](https://github.com/Qazaroth/feckmania/commit/37cf2c2f10e0b679c26c785a7000e2c2aca8d179))

- Filtered queries can now be exported into files
([`e4080b4…`](https://github.com/Qazaroth/feckmania/commit/e4080b48a8bcd824deb6c69f0e3c882a3c073a51))

## [0.3.0] - 2026-08-20

### 🚀 Features

- Basic CLI interface implemented
([`633c964…`](https://github.com/Qazaroth/feckmania/commit/633c964a156073d6993c05f76f580e7b0410df2f))


### 🚜 Refactor

- Split main.rs into smaller modular files
([`f497d4d…`](https://github.com/Qazaroth/feckmania/commit/f497d4d284ec99cf54b4dfdb35137c7536877bdb))


### ⚙️ Continuous Integration

- Release workflow added
([`5a8ac5d…`](https://github.com/Qazaroth/feckmania/commit/5a8ac5d509ab6c9171cca0300bd7df34ef87edef))

## [0.2.0] - 2026-08-20

### 🚀 Features

- 'EventRecord' struct added
([`297b452…`](https://github.com/Qazaroth/feckmania/commit/297b4520ea8ceb9cadd4bb587aee76af271767ea))

- XML-to-struct parser implemented and works as intended
([`16fd6ce…`](https://github.com/Qazaroth/feckmania/commit/16fd6cedd982ea1e48fa9dab2844a0568e9e52d6))

## [0.1.0] - 2026-08-20

### 🚀 Features

- Wrapper for EvtQuery and EvtNext implemented
([`3856f1b…`](https://github.com/Qazaroth/feckmania/commit/3856f1bdc275999f41b9414baf44418ef7b9b0f5))

- Able to now read active live channels
([`dbe05b6…`](https://github.com/Qazaroth/feckmania/commit/dbe05b65f2daec411b851fdcdf6cdf62aca66827))

- Parsing offline static files implemented
([`4548de7…`](https://github.com/Qazaroth/feckmania/commit/4548de7264ac3c14235b2332bd82c877265c6c49))


### 🧹 Chores

- Initial commit
([`96761a5…`](https://github.com/Qazaroth/feckmania/commit/96761a5ef179a5b8691f27e4ff47eefdef67f734))

