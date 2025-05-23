use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

// Represents the database to benchmark
#[derive(Clone, Copy, PartialEq)]
pub enum Database {
    HelixDB,
    Neo4j,
}

// Configuration for the benchmark
#[derive(Clone)]
pub struct Benchmark {
    pub database: Database,
    pub endpoint: Option<String>,
}

// Parameters for scan operations
#[derive(Clone)]
pub struct Scan {
    pub limit: Option<usize>,
    pub start: Option<usize>,
    projection: Projection,
}

#[derive(Clone, Copy)]
pub enum Projection {
    Id,
    Full,
    Count,
}

impl Scan {
    pub fn new(limit: Option<usize>, start: Option<usize>, projection: Projection) -> Self {
        Self {
            limit,
            start,
            projection,
        }
    }

    pub fn projection(&self) -> Result<Projection> {
        Ok(self.projection)
    }
}

#[async_trait]
pub trait BenchmarkClient {
    async fn startup(&self) -> Result<()>;
    async fn create_records(&mut self, count: usize) -> Result<()>;
    async fn read_records(&self) -> Result<()>;
    async fn update_records(&self) -> Result<()>;
    async fn delete_records(&self) -> Result<()>;
    async fn scan_records(&self) -> Result<()>;
    async fn count_records(&self) -> Result<usize>;

    //async fn bulk_create_string(&self, count: usize, val: Value) -> Result<()>;
    //async fn huge_traversal(&self, count: usize) -> Result<()>;
}

#[async_trait]
pub trait BenchmarkEngine {
    async fn setup(options: &Benchmark) -> Result<Self>
    where
        Self: Sized;
    async fn create_client(&self) -> Result<Box<dyn BenchmarkClient>>;
}
