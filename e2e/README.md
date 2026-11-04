# Native E2E

This directory covers first launch, workspace creation, restart persistence, and the
network-disabled Practice loop.
It drives the release Tauri binary through the W3C WebDriver boundary, so the React UI,
Tauri IPC commands, and SQLite persistence all participate.

## Linux prerequisites

- `cargo install tauri-driver --version 2.0.6 --locked`
- Ubuntu 26.04: `sudo apt-get install webkitgtk-webdriver xvfb`
- Ubuntu 22.04/24.04 package name: `webkit2gtk-driver`
- `unshare` (util-linux) and `ip` (iproute2), with user/network namespaces permitted
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

## Offline Practice acceptance

`offline-practice-loop.test.mjs` is included automatically by both existing e2e scripts.
It re-executes only itself using `unshare --user --map-root-user --net`, brings `lo` up,
and launches the Node harness, tauri-driver, WebKit driver and release app inside that
private network namespace. No physical interfaces, virtual uplinks or host firewall rules
are changed. The other native tests and the release build run outside isolation, so build
dependencies can still download. This is Linux-only and fails on unsupported hosts.

Before app launch and after the loop, the guard checks a different `/proc/self/ns/net`
identity from the parent, exactly one interface (`lo`), no routes outside loopback in any
IPv4/IPv6 routing table, immediate `ENETUNREACH` connecting to literal external IPv4 and
IPv6 addresses, and a successful TCP connection to a temporary loopback listener. A timeout,
permission failure, missing command or forged marker is a failure, never a skip. Literal
addresses avoid confusing DNS failures with isolation. All native processes inherit the
namespace; WebDriver still communicates over loopback. This proves this flow can complete
without external networking, not that the app never attempts best-effort network calls.
Filesystem Unix-domain sockets to host services remain accessible; this is network
namespace isolation, not a general IPC sandbox.

The child writes a parent-created completion marker only after all learner assertions and
the final isolation guard pass. A zero exit without that marker fails even if nested Node
test discovery ran no tests. The outer test allows 150 seconds, the inner flow 120; aborting
the outer test terminates the isolated process group and force-kills survivors.
Forced termination can bypass the learner test's `finally`, leaving its temporary
`XDG_DATA_HOME` on a local machine. Descendants that create a separate session can escape
the process group; this harness does not provide cgroup containment. CI uses disposable VMs.

The test creates real learner data through the UI, opens the bundled vertical-axis Shell
concept (its full provider-owned name), starts Practice, rejects `0`, verifies the automatic
hint, requests another hint, solves the visible generated polynomial problem with an
independent shell-integral calculation, and advances to Problem 2 with cleared answer,
hints and feedback. It uses no sample import, mocked IPC, database seeding or canonical
solution lookup. Problem 2 may legitimately have the same prompt as Problem 1.

For an already built release binary, run:

```sh
xvfb-run -a node --test e2e/offline-practice-loop.test.mjs
node --test e2e/harness-checks.mjs
```

WSL2 uses the same namespace mechanism. WSLg can use the first command without `xvfb-run`
if its display works inside the namespace; Xvfb is more reproducible. Install the matching
WebKit driver and ensure `tauri-driver`, `WebKitWebDriver`, `ip` and `unshare` are on PATH.
`AXIOM_E2E_APP` and `TAURI_DRIVER_BIN` work as for the other tests. Do not manually set
`AXIOM_E2E_PARENT_NETNS`; the wrapper supplies it and the guard verifies kernel state.

Ubuntu runners can deny unprivileged user namespaces via AppArmor. The CI e2e job explicitly
sets `kernel.apparmor_restrict_unprivileged_userns=0` on its disposable VM. On a restricted
local machine, use an environment that permits namespaces; the harness never silently
falls back to an online run or changes this system setting itself.

For a negative offline-behavior check, use a disposable source copy: temporarily await
`fetch('https://1.1.1.1/', { mode: 'no-cors', signal: AbortSignal.timeout(5000) })` before
`describeAttempt` invokes Practice. Build that copy and save its binary separately. Run
the same learner assertions once online with only the namespace wrapper/guards/marker
removed from a disposable test copy; require a pass to exclude CORS/TLS/endpoint failures.
Then run the unmodified isolated test against that exact binary; require the Answer wait
to fail. Rebuild clean source, save a distinct clean binary, and rerun all native flows.
A failed namespace guard alone is not this negative Practice proof. The task 068 handoff
records which of these checks were actually executed.
