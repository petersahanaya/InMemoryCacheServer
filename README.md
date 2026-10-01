# Nori Database

Nori is a custom in-memory cache server built with Rust. The system provides a simple command-based interface over TCP for storing, retrieving, deleting, and monitoring cached data.

The project is developed as part of a final-year thesis project with a focus on designing and implementing a TCP-based cache server using Rust.

## Features

- TCP-based client-server communication
- In-memory data storage using `HashMap`
- `GET` command for retrieving data
- `SET` command for storing data
- `SET ... EX` for storing data with a time-to-live (TTL)
- `SETX` command as an alternative TTL-based storage command
- `DELETE` / `DEL` command for removing data
- `STATS` command for cache statistics
- Cache hit and cache miss tracking
- Automatic expiration of TTL-based data
- Background expired-cache sweeping
- Data persistence to `storage.cache`
- JSON serialization using Serde
- Zstandard compression for persisted data
- Interactive command-line client using Rustyline
- Asynchronous TCP networking using Tokio

## Architecture

```text
┌─────────────────────┐
│    Nori CLI Client  │
│     Rustyline       │
└──────────┬──────────┘
           │
           │ TCP
           ▼
┌─────────────────────┐
│     Network Layer   │
│   Tokio TCP Server  │
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│    Parser Layer     │
│  Command Validation │
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│    Storage Layer    │
│      HashMap        │
│                     │
│ GET / SET / DELETE  │
│       / STATS       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Persistence Layer   │
│   serde_json + zstd │
└──────────┬──────────┘
           ▼
      storage.cache
```

## Technology Stack

| Component | Technology |
|---|---|
| Programming Language | Rust |
| Async Runtime | Tokio |
| Network Protocol | TCP |
| Data Structure | `HashMap<String, Data>` |
| Serialization | Serde / serde_json |
| Compression | Zstandard (`zstd`) |
| CLI Interface | Rustyline |
| Persistence | `storage.cache` |

## Requirements

Make sure the following tools are installed:

- Rust toolchain with Cargo
- A terminal environment capable of running the Rust binaries

You can verify the Rust installation with:

```bash
rustc --version
cargo --version
```

## Project Structure

```text
.
├── Cargo.toml
├── Cargo.lock
├── README.md
├── project.md
├── storage.cache
├── nori_history.txt
└── src
    ├── main.rs
    ├── bin
    │   └── client.rs
    └── layers
        ├── mod.rs
        ├── network_layer.rs
        ├── parser_layer.rs
        └── storage_layer.rs
```

## Running the Server

Start the Nori server with:

```bash
cargo run --bin nori-server
```

The server listens on:

```text
127.0.0.1:6379
```

When the server starts, it attempts to load persisted data from `storage.cache`.

## Running the CLI Client

Open another terminal while the server is running and execute:

```bash
cargo run --bin nori
```

The client connects to:

```text
127.0.0.1:6379
```

After connecting, the prompt is displayed as:

```text
nori>
```

## Available Commands

### GET

Retrieve a value using its key:

```text
GET username
```

### SET

Store a value without expiration:

```text
SET username peter
```

### SET with TTL

Store a value with an expiration time in seconds:

```text
SET username peter EX 60
```

The value is configured to expire after 60 seconds.

### SETX

Alternative syntax for storing data with TTL:

```text
SETX username peter 60
```

### DELETE

Delete a key:

```text
DELETE username
```

The shorter alias is also supported:

```text
DEL username
```

### STATS

Display cache statistics:

```text
STATS
```

The response includes:

- Cache hits
- Cache misses
- Total keys currently stored

### HELP

Display the available server commands:

```text
HELP
```

## Local CLI Commands

The Nori CLI also provides commands that are handled locally and are not sent to the server.

```text
help
clear
exit
quit
```

- `help` — displays the CLI and server command reference
- `clear` — clears the terminal screen
- `exit` / `quit` — closes the client

## Response Format

The server returns JSON responses. For example, a successful `SET` operation returns a response similar to:

```json
{
  "success": true,
  "message": "Data stored successfully.",
  "data": null
}
```

A successful `GET` operation returns the stored data:

```json
{
  "success": true,
  "message": "Data found.",
  "data": {
    "value": "peter",
    "expired_at": null
  }
}
```

## TTL and Expiration

Each stored value can optionally have an expiration timestamp.

The internal data structure is conceptually represented as:

```text
Data
├── value: String
└── expired_at: Option<u64>
```

For data without TTL, `expired_at` is `null`.

For data with TTL, the server calculates an expiration timestamp based on the current Unix timestamp and the configured TTL.

Expired data is handled in two ways:

1. When a key is requested with `GET`, the server checks whether the data has expired. If it has expired, the key is removed and the request is counted as a cache miss.
2. A background task periodically scans the storage and removes expired entries.

The background expiration sweep currently runs every 10 seconds.

## Persistence

Nori persists the current storage to:

```text
storage.cache
```

The persistence process is:

```text
HashMap
   │
   ▼
serde_json
   │
   ▼
JSON bytes
   │
   ▼
zstd compression
   │
   ▼
storage.cache
```

When the server starts, the persistence file is loaded and decompressed. Expired entries are removed before the storage becomes available to clients.

The server also uses a temporary file during persistence before replacing the existing storage file.

## Cache Statistics

Nori tracks two cache access statistics during the server runtime:

```text
cache_hits
cache_misses
```

A successful lookup of an existing, non-expired key increments `cache_hits`.

A lookup for a missing or expired key increments `cache_misses`.

These runtime counters are not persisted between server restarts.

## Development

Build the project with:

```bash
cargo build
```

Run the server:

```bash
cargo run --bin nori-server
```

Run the client:

```bash
cargo run --bin nori
```

For a release build:

```bash
cargo build --release
```

The compiled binaries are generated under:

```text
target/release/
```

## Thesis Context

This project is intended to support the research and implementation of a custom cache server with TCP-based communication.

The current project title is:

> **Perancangan dan Implementasi Cache Server Berbasis TCP dengan Dukungan Time-To-Live dan Data Persistence Menggunakan Rust**

The implementation focuses on the design and realization of the server, including TCP communication, command parsing, in-memory storage, TTL management, cache statistics, and data persistence.

## Scope

The current implementation focuses on functional implementation of the cache server and its supporting components. It does not aim to provide compatibility with the Redis protocol or Redis clients.

The server uses its own command syntax and response format.
