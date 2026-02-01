# MCP Daddy Monorepo

This repository is organized as a small monorepo.

Important:

- The git repo root is `code/`.
- `plans/` is planning material and is not intended to be committed.

## Layout

- `apps/`
  - `apps/web/`: Marketing/landing website.
  - `apps/mcp-daddy-app/`: Desktop app (Tauri + React).
- `crates/`: Rust library crates shared across the project.
- `bin/`: Rust binaries/CLIs (e.g. the stdio bridge).

## Working Conventions

- Only code changes under `code/` are intended to be committed/pushed.
- Plans and discussions live under `plans/` at the workspace root.

## Commands

Run these from the monorepo root (`code/`):

- `pnpm dev`: start the landing page dev server
- `pnpm lint`: lint the landing page
- `pnpm typecheck`: run TypeScript typechecking
- `pnpm build`: build the landing page
- `pnpm format`: format files under `code/`
