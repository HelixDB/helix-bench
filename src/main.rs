use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::json;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};
use tokio;

mod helixdb;
mod neo4j;
mod types;
mod utils;

use crate::types::BenchmarkEngine;
use crate::helixdb::HelixDBEngine;
use crate::neo4j::Neo4jEngine;
use crate::types::{Benchmark, BenchmarkClient, Database, KeyType, Projection, Scan};

#[derive(Parser)]
#[command(name = "helix-bench")]
#[command(about = "Benchmarking tool for HelixDB")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Benchmark a specific operation
    Bench {
        /// Operation to benchmark: create, read, update, delete, scan
        #[arg(default_value = "all")]
        operation: String,
        /// Number of operations to perform
        #[arg(short, long, default_value_t = 1000)]
        count: usize,
        /// Key type: u32 or string
        #[arg(short, long, default_value = "u32")]
        key_type: String,
        /// Database: helixdb, neo4j or others
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
            let pb = ProgressBar::new(count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Create")
                .unwrap()
                .progress_chars("##-"),
            );
            for i in 0..count as u32 {
                client.create_u32(i, sample_value.clone()).await?;
                pb.inc(1);
            }
            pb.finish_with_message("Create complete");
        }
        ("create", KeyType::String) => {
            let pb = ProgressBar::new(count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Create")
                .unwrap()
                .progress_chars("##-"),
            );
            for i in 0..count {
                let key = format!("key{}", i);
                client.create_string(key, sample_value.clone()).await?;
                pb.inc(1);
            }
            pb.finish_with_message("Create complete");
        }
        ("read", KeyType::U32) => {
            let pb = ProgressBar::new(count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Read")
                .unwrap()
                .progress_chars("##-"),
            );
            for i in 0..count as u32 {
                client.read_u32(i).await?;
                pb.inc(1);
            }
            pb.finish_with_message("Read complete");
        }
        ("read", KeyType::String) => {
            let pb = ProgressBar::new(count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Read")
                .unwrap()
                .progress_chars("##-"),
            );
            for i in 0..count {
                let key = format!("key{}", i);
                client.read_string(key).await?;
                pb.inc(1);
            }
            pb.finish_with_message("Read complete");
        }
        ("update", KeyType::U32) => {
            let pb = ProgressBar::new(count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Update")
                .unwrap()
                .progress_chars("##-"),
            );
            let updated_value = json!({"data": "updated_value"});
            for i in 0..count as u32 {
                client.update_u32(i, updated_value.clone()).await?;
                pb.inc(1);
            }
            pb.finish_with_message("Update complete");
        }
        ("update", KeyType::String) => {
            let pb = ProgressBar::new(count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Update")
                .unwrap()
                .progress_chars("##-"),
            );
            let updated_value = json!({"data": "updated_value"});
            for i in 0..count {
                let key = format!("key{}", i);
                client.update_string(key, updated_value.clone()).await?;
                pb.inc(1);
            }
            pb.finish_with_message("Update complete");
        }
        ("delete", KeyType::U32) => {
            let pb = ProgressBar::new(count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Delete")
                .unwrap()
                .progress_chars("##-"),
            );
            for i in 0..count as u32 {
                client.delete_u32(i).await?;
                pb.inc(1);
            }
            pb.finish_with_message("Delete complete");
        }
        ("delete", KeyType::String) => {
            let pb = ProgressBar::new(count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Delete")
                .unwrap()
                .progress_chars("##-"),
            );
            for i in 0..count {
                let key = format!("key{}", i);
                client.delete_string(key).await?;
                pb.inc(1);
            }
            pb.finish_with_message("Delete complete");
        }
        ("scan", KeyType::U32 | KeyType::String) => {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] Running scan...")
                .unwrap(),
            );
            pb.enable_steady_tick(Duration::from_millis(100));
            let scan = Scan::new(Some(count), None, Projection::Full);
            client.scan_u32(&scan).await?;
            pb.finish_with_message("Scan complete");
        }
        _ => return Err(anyhow::anyhow!("Unsupported operation: {}", operation)),
    }

    Ok(start.elapsed())
}

async fn run_all_benchmarks(
    client: &dyn BenchmarkClient,
    count: usize,
    key_type: KeyType,
) -> Result<Vec<(String, Duration)>> {
    let operations = vec!["create", "read", "update", "delete", "scan"];
    let mut results = Vec::new();

    let create_duration = run_benchmark(client, "create", count, key_type).await?;
    results.push(("create".to_string(), create_duration));

    let read_duration = run_benchmark(client, "read", count, key_type).await?;
    results.push(("read".to_string(), read_duration));

    // TODO: throwing error (connection closed before message completed) on helixdb
    //let update_duration = run_benchmark(client, "update", count, key_type).await?;
    //results.push(("update".to_string(), update_duration));

    let delete_duration = run_benchmark(client, "delete", count, key_type).await?;
    results.push(("delete".to_string(), delete_duration));

    let scan_duration = run_benchmark(client, "scan", count, key_type).await?;
    results.push(("scan".to_string(), scan_duration));

    Ok(results)
}

fn database_name(database: Database) -> &'static str {
    match database {
        Database::HelixDB => "HelixDB",
        Database::Neo4j => "Neo4j",
    }
}

fn key_type_name(key_type: KeyType) -> &'static str {
    match key_type {
        KeyType::U32 => "u32",
        KeyType::String => "string",
    }
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

            if operation.to_lowercase() == "all" {
                let results = run_all_benchmarks(&*client, count, key_type).await?;
                println!(
                    "\nBenchmark Results for {} ({} operations, key type: {}):",
                    database_name(database),
                    count,
                    key_type_name(key_type)
                );
                println!("{:-<50}", "");
                println!("{:<10} | {:<15} | {:<15}", "Operation", "Duration", "Ops/s");
                println!("{:-<50}", "");
                for (op, duration) in results {
                    println!(
                        "{:<10} | {:<15} | {:<15.2}",
                        op,
                        format!("{:?}", duration),
                        count as f64 / duration.as_secs_f64()
                    );
                }
            } else {
                let duration = run_benchmark(&*client, &operation, count, key_type).await?;
                println!(
                    "Benchmark: {} {} operations on {} took {:?} ({:.2} ops/s)",
                    operation,
                    count,
                    database_name(database),
                    duration,
                    count as f64 / duration.as_secs_f64()
                );
            }}
    }

    Ok(())
}
