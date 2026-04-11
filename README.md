# stars-reborn-engine

Game engine for **Stars Reborn** — an open-source remake of the classic 4X
space strategy game *Stars!* (1996).

## What this is

The engine is a standalone Rust HTTP server. It is the single source of truth
for all game rules, data structures, and turn processing. The UI layer
(`stars-reborn-ui`) is a pure HTTP client and never implements game logic.

## API

### Data model (`/model/`) — read-only, static game rules

```
GET /model/version                      engine version + API schema version
GET /model/technologies                 full technology tree
GET /model/technologies/{id}            single technology item
GET /model/race/traits                  PRT/LRT definitions and point costs
GET /model/ships/hulls                  all hull definitions
GET /model/schemas                      JSON schemas for all request/response types
```

### Game (`/games/`) — stateful gameplay

```
POST /games                                           create new game
GET  /games/{game_id}                                 game metadata
GET  /games/{game_id}/turns/{year}/status             per-player submission status
GET  /games/{game_id}/turns/{year}/players/{pid}      turn file for player
PUT  /games/{game_id}/turns/{year}/orders/{pid}       submit player orders (idempotent)
POST /games/{game_id}/turns/{year}/skip/{pid}         generate AI orders, mark skipped
GET  /games/{game_id}/turns/{year}/ai-orders/{pid}    AI-suggested orders (not submitted)
```

For full API contracts, mechanic documentation, and architecture decisions see
[stars-reborn-design](https://github.com/stars-reborn/stars-reborn-design).

## Building

```sh
cargo build
cargo build --release
```

## Running

```sh
# Default: binds to 127.0.0.1:8080
RUST_LOG=info cargo run

# Custom address
SERVER_ADDR=0.0.0.0:9000 RUST_LOG=debug cargo run
```

## Testing

```sh
cargo test
```

## Design

Architecture, mechanics, and API contracts are documented in
`stars-reborn-design`. Consult that repo before modifying engine behavior —
it is the authoritative developer reference.

## License

MIT — see [LICENSE](LICENSE).
