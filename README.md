## HelixDB Benchmarking Suite

### Getting Started
```bash
cargo run -- bench --database helixdb
cargo run -- bench --database neo4j
```

### Resuts
#### HelixDB
Benchmark Results for HelixDB (10000 operations, key type: u32):
--------------------------------------------------
Operation  | Duration        | Ops/s
--------------------------------------------------
create     | 48.364199806s   | 206.76
read       | 6.066777425s    | 1648.32
delete     | 4.321698054s    | 2313.91
scan       | 52.043912ms     | 192145.43

#### Neo4j (running in docker)
Benchmark Results for Neo4j (10000 operations, key type: u32):
--------------------------------------------------
Operation  | Duration        | Ops/s
--------------------------------------------------
create     | 45.125594234s   | 221.60
read       | 33.523005344s   | 298.30
delete     | 47.519269653s   | 210.44
scan       | 26.505164ms     | 377284.97
