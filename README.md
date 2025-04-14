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
create     | 47.189923905s   | 211.91
read       | 5.03537907s     | 1985.95
delete     | 4.444712907s    | 2249.86
scan       | 103.556229ms    | 96565.90

#### Neo4j (running in docker)
Benchmark Results for Neo4j (10000 operations, key type: u32):
--------------------------------------------------
Operation  | Duration        | Ops/s
--------------------------------------------------
create     | 37.952741282s   | 263.49
read       | 30.528510312s   | 327.56
delete     | 45.759882508s   | 218.53
scan       | 12.016267ms     | 832205.21
