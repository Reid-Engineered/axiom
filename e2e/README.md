# Native E2E

This directory holds the one Stage 7 native flow: first launch → create workspace → home.
It drives the release Tauri binary through the W3C WebDriver boundary, so the React UI,
Tauri IPC commands, and SQLite persistence all participate.

## Linux prerequisites

- `cargo install tauri-driver --version 2.0.6 --locked`
- Ubuntu 26.04: `sudo apt-get install webkitgtk-webdriver xvfb`
- Ubuntu 22.04/24.04 package name: `webkit2gtk-driver`
- Project dependencies installed with `npm ci`

Run `npm run test:e2e:linux`. The script builds the unbundled release binary, starts the
test under Xvfb, gives the app an isolated temporary `XDG_DATA_HOME`, and removes that data
afterward. WSLg sessions with a working display can use `npm run test:e2e` directly.

`TAURI_DRIVER_BIN` can point to a non-PATH `tauri-driver`; `AXIOM_E2E_APP` can point to an
already-built binary. The native WebKit driver itself must be on `PATH` so `tauri-driver`
can launch it.

The client is the exact-pinned `selenium-webdriver` package. Playwright cannot attach to
Tauri's native W3C WebDriver transport, and a full WebdriverIO/plugin installation would be
disproportionate for one flow. On WebKitGTK 2.52, typing uses the W3C Actions endpoint to
avoid tauri-driver 2.0.6's known legacy Element Send Keys forwarding defect.

For Linux CI, install the Tauri build prerequisites plus the two driver packages above,
cache Cargo and `src-tauri/target`, and run `npm run test:e2e:linux`. A cold release build is
the expensive part; the flow itself is intentionally narrow.
