use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

#[derive(Clone, Copy, PartialEq)]
enum KeyType {
    U32,
    String,
}

// Represents the database to benchmark
#[derive(Clone, Copy, PartialEq)]
enum Database {
    HelixDB,
    Neo4j,
}

// Configuration for the benchmark
#[derive(Clone)]
struct Benchmark {
    database: Database,
    endpoint: Option<String>,
}

// Parameters for scan operations
#[derive(Clone)]
struct Scan {
    limit: Option<usize>,
    start: Option<usize>,
    projection: Projection,
}

#[derive(Clone, Copy)]
enum Projection {
    Id,
    Full,
    Count,
}

impl Scan {
    fn new(limit: Option<usize>, start: Option<usize>, projection: Projection) -> Self {
        Self {
            limit,
            start,
            projection,
        }
    }

    fn projection(&self) -> Result<Projection> {
        Ok(self.projection)
    }
}

// Trait defining benchmark operations
#[async_trait]
trait BenchmarkClient {
    async fn startup(&self) -> Result<()>;
    async fn create_u32(&self, key: u32, val: Value) -> Result<()>;
    async fn create_string(&self, key: String, val: Value) -> Result<()>;
    async fn read_u32(&self, key: u32) -> Result<()>;
    async fn read_string(&self, key: String) -> Result<()>;
    async fn update_u32(&self, key: u32, val: Value) -> Result<()>;
    async fn update_string(&self, key: String, val: Value) -> Result<()>;
    async fn delete_u32(&self, key: u32) -> Result<()>;
    async fn delete_string(&self, key: String) -> Result<()>;
    async fn scan_u32(&self, scan: &Scan) -> Result<usize>;
    async fn scan_string(&self, scan: &Scan) -> Result<usize>;
}

// Trait for setting up the benchmark environment
#[async_trait]
trait BenchmarkEngine {
    async fn setup(options: &Benchmark) -> Result<Self>
    where
        Self: Sized;
    async fn create_client(&self) -> Result<Box<dyn BenchmarkClient>>;
}
