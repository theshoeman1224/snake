## Components

### Game Engine

`src/game.rs` owns the board, snake segments, direction, food, score, movement,
growth, and collision rules. `src/config.rs` defines preset difficulties and
bounded custom settings. These modules compile natively for unit testing and do
not depend on browser APIs.

### Browser Controller

`src/web.rs` initializes the WASM application and coordinates game state through
an `Rc<RefCell<App>>`. It installs keyboard, pointer, mode, pause, restart, and
focus listeners; advances fixed simulation ticks from animation-frame
timestamps; and updates status overlays and build information.

### Canvas Renderer

`src/renderer.rs` maps logical board cells to an HTML Canvas. It draws the arena,
food, snake head, and body with distinct colors while leaving all gameplay
decisions in the engine.

### Web Shell

`index.html` supplies semantic controls and live regions. `styles/game.css`
provides the terminal-inspired visual system, responsive layout, mobile D-pad,
game overlays, and reduced-motion behavior.

## Runtime Flow

1. The WASM start function resolves required DOM elements and installs event listeners.
2. Selecting a mode creates a new `Game` and starts a three-second countdown.
3. `requestAnimationFrame` supplies render timestamps to the browser controller.
4. The controller accumulates elapsed time and advances the engine at the selected movement rate.
5. The Canvas renderer draws the latest game state, while DOM elements report score and status.
6. Build metadata is displayed locally and compared with the public GitHub `main` commit when available.

## Build Flow

`build.rs` embeds the source commit into the Rust artifact. Trunk compiles the
`cdylib` to WebAssembly, generates JavaScript bindings, optimizes the bundle, and
uses relative asset URLs. `scripts/build-web.sh` adds `dist/game-manifest.json`,
and `scripts/export-to-resume.sh` copies the static bundle into a portfolio
repository.

GitHub Actions runs formatting, nine native unit tests, native Clippy, WASM
Clippy, and an optimized Trunk build. External actions are pinned to immutable
commit SHAs.
