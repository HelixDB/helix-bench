use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

// CLI configuration
#[derive(Parser)]
#[command(name = "helixdb-bench")]
#[command(about = "Benchmarking tool for HelixDB and Neo4j")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Benchmark a specific operation
    Bench {
        /// Operation to benchmark: create, read, update, delete, scan
        operation: String,
        /// Number of operations to perform
        #[arg(short, long, default_value_t = 1000)]
        count: usize,
        /// Key type: u32 or string
        #[arg(short, long, default_value = "u32")]
        key_type: String,
        /// Database: helixdb or neo4j
        #[arg(short, long, default_value = "helixdb")]
        database: String,
        /// Endpoint URL (optional)
        #[arg(short, long)]
        endpoint: Option<String>,
    },
}

async fn run_benchmark(
    client: &dyn BenchmarkClient,
    operation: &str,
    count: usize,
    key_type: KeyType,
) -> Result<Duration> {
    let start = Instant::now();
    let sample_value = json!({"data": "test_value"});

    match (operation.to_lowercase().as_str(), key_type) {
        ("create", KeyType::U32) => {
            for i in 0..count as u32 {
                client.create_u32(i, sample_value.clone()).await?;
            }
        }
        ("create", KeyType::String) => {
            for i in 0..count {
                let key = format!("key{}", i);
                client.create_string(key, sample_value.clone()).await?;
            }
        }
        ("read", KeyType::U32) => {
            for i in 0..count as u32 {
                client.read_u32(i).await?;
            }
        }
        ("read", KeyType::String) => {
            for i in 0..count {
                let key = format!("key{}", i);
                client.read_string(key).await?;
            }
        }
        ("update", KeyType::U32) => {
            let updated_value = json!({"data": "updated_value"});
            for i in 0..count as u32 {
                client.update_u32(i, updated_value.clone()).await?;
            }
        }
        ("update", KeyType::String) => {
            let updated_value = json!({"data": "updated_value"});
            for i in 0..count {
                let key = format!("key{}", i);
                client.update_string(key, updated_value.clone()).await?;
            }
        }
        ("delete", KeyType::U32) => {
            for i in 0..count as u32 {
                client.delete_u32(i).await?;
            }
        }
        ("delete", KeyType::String) => {
            for i in 0..count {
                let key = format!("key{}", i);
                client.delete_string(key).await?;
            }
        }
        ("scan", KeyType::U32 | KeyType::String) => {
            let scan = Scan::new(Some(count), None, Projection::Full);
            client.scan_u32(&scan).await?;
        }
        _ => return Err(anyhow::anyhow!("Unsupported operation: {}", operation)),
    }

    Ok(start.elapsed())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bench {
            operation,
            count,
            key_type,
            database,
            endpoint,
        } => {
            let key_type = match key_type.to_lowercase().as_str() {
                "u32" => KeyType::U32,
                "string" => KeyType::String,
                _ => return Err(anyhow::anyhow!("Invalid key type: {}", key_type)),
            };

            let database = match database.to_lowercase().as_str() {
                "helixdb" => Database::HelixDB,
                "neo4j" => Database::Neo4j,
                _ => return Err(anyhow::anyhow!("Invalid database: {}", database)),
            };

            let options = Benchmark { database, endpoint };
            let engine: Box<dyn BenchmarkEngine> = match database {
                Database::HelixDB => Box::new(HelixDBEngine::setup(&options).await?),
                Database::Neo4j => Box::new(Neo4jEngine::setup(&options).await?),
            };

            let client = engine.create_client().await?;
            let duration = run_benchmark(&*client, &operation, count, key_type).await?;

            println!(
                "Benchmark: {} {} operations on {} took {:?} ({:.2} ops/s)",
                operation,
                count,
                database_name(database),
                duration,
                count as f64 / duration.as_secs_f64()
            );
        }
    }

    Ok(())
}

fn database_name(database: Database) -> &'static str {
    match database {
        Database::HelixDB => "HelixDB",
        Database::Neo4j => "Neo4j",
    }
}
