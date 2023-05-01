# Tawatcher

GUI for [activity watchers](https://github.com/2e3s/awatcher) ([ActivityWatch](https://github.com/ActivityWatch) server).
It is mostly a Tauri (a prominent alternative to Electron) wrapper around the aforementioned projects, and it uses WebKit to render the UI.
This application has a tenfold higher memory footprint, so it may be better to use the [watchers](https://github.com/2e3s/awatcher) directly.


## Execution

### Build prerequisites

- Rust nighly toolchain
- The submodule `aw-webui` should be initialized, and the `dist` build by `npm run build` inside the submodule
- Tauri [prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites/#setting-up-linux)

### Run from sources

In the root directory:
```sh
npm install
cargo install tauri
cargo tauri dev
```

### Build

`cargo tauri build` or download the published executable with the latest release.
Right now, `AppImage` is not included because windows cannot be tracked.

## Configuration

See the [corresponding section](https://github.com/2e3s/awatcher#configuration) of `awatcher` for file configuration and window title filters.
