## HelixDB Benchmarking Suite

### Getting Started
Setup
```bash
helix deploy --local --path "helixdb-queries/"
sh start_neo4j.sh
```
Benchmarking
```bash
cargo run -- bench --database helixdb
cargo run -- bench --database neo4j
```

### Resuts
CPU: AMD Ryzen 7 PRO 6850U @ 2.701GHz, RAM: 32GB @ 6400 MHz
#### HelixDB (faulty because of create and update queries)
```
Benchmark Results for HelixDB (10000 operations, key type: u32):
--------------------------------------------------------------------------------
Operation  | Total Time      | Avg Time/Req (ms) | Throughput (ops/s)
--------------------------------------------------------------------------------
create     | 48.553177931s   | 4.855318        | 205.96
read       | 4.540390305s    | 0.454039        | 2202.45
delete     | 4.987414496s    | 0.498741        | 2005.05
scan       | 83.003203ms     | 0.008300        | 120477.28
```

#### Neo4j (running in docker)
```
Benchmark Results for Neo4j (10000 operations, key type: u32):
--------------------------------------------------------------------------------
Operation  | Total Time      | Avg Time/Req (ms) | Throughput (ops/s)
--------------------------------------------------------------------------------
create     | 46.271207132s   | 4.627121        | 216.12
read       | 31.89390607s    | 3.189391        | 313.54
update     | 59.162798138s   | 5.916280        | 169.03
delete     | 49.311293407s   | 4.931129        | 202.79
scan       | 24.825754ms     | 0.002483        | 402807.50
```
