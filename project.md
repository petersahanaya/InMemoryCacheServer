# Overall Project flow

Client
  │
  │ JSON request
  ▼
TCP Server
  │
  ▼
Command Parser
  │
  ▼
HashMap Cache
  │
  ├── GET
  ├── SET
  ├── DELETE
  └── STATS
  │
  ▼
Persistence Layer
  │
  ├── serde_json
  └── zstd
       │
       ▼
  storage.cache

# Persistense flow
In-memory:
HashMap<String, Data>

Serialization:
serde + serde_json

Persistence:
storage.cache

Compression:
zstd

Protocol:
JSON over TCP
