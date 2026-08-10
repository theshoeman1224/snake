## Overview

Snake is a browser game written in Rust and compiled to WebAssembly. It was
developed as an interactive project that can be embedded in a static engineering
portfolio without requiring an application server.

## Motivation

The project started as a terminal application, then moved to the browser after
the game rules were extracted into a reusable Rust library. That progression
kept the gameplay model independent from its presentation while making the
result easier to distribute and play from a resume website.

## Key Engineering Challenges

Browser rendering and game movement run at different rates. The frontend uses
`requestAnimationFrame` for visual updates and a fixed-step accumulator for
movement, with delayed frames capped to prevent a burst of simulation updates
after a browser stall. Direction input is queued once per game tick, and the
engine rejects immediate reversals.

Collision handling distinguishes the body from a tail cell that will vacate on
the current tick. This allows a legal move into that cell without weakening
self-collision detection, and a unit test protects the behavior.

## Player Experience

Easy, Medium, and Hard modes provide predefined arena sizes and movement speeds.
Custom mode exposes bounded width, height, and speed settings. Players can use
Arrow keys, WASD, or a responsive on-screen D-pad, and the game pauses when the
browser loses focus. Mode changes and restarts begin with a three-second
countdown.

## Build And Delivery

Trunk produces relative static assets suitable for deployment below an
arbitrary website path. The release script requires a clean worktree, embeds the
checked-out Git commit into the build, and writes a JSON manifest with the
repository, commit, and build time. A separate export script copies the bundle
into a resume project's public assets for iframe-based integration.

The repository currently provides a local WASM demo build but does not publish a
hosted demo. A real gameplay screenshot also remains to be captured.
