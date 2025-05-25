## Benchmarks
All tests were run on a MacBook Air M3

### HelixDB (via http server)
#### Graph Only Ops
```
Benchmark Results for HelixDB (10000 operations):
-------------------------------------------------
Operation  | Total Time      | Avg Time/Req (ms)
-------------------------------------------------
create     | 52.791787s      | 5.279179
read       | 3.060539084s    | 0.306054
update     | 53.162571583s   | 5.316257
scan       | 134.95025ms     | 0.013495
```

- batch_insert (total)
#### Vector Ops
- insert (total, per insert)
- search (total, per search)

### HelixDB (embedded)
#### Graph Only Ops
- insert (total, per insert)
- batch_insert (total)
- read (total, per read)
- update (total, per update)
- scan (total, per op)

#### Vector Ops
```
Total insertion time for 100_000 vectors: 23.965mins
Average insertion time per query (measured individually): 14.374621ms
Storage space size: 1747 MB (1.747 GB)
```
```
Total search time for 5000 queries, at k = 12: 55.888562136s
Average time per search (total/num_vectors): 11.181567ms
Average search time per query (measured individually): 11.177712ms
```
