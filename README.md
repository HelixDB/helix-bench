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