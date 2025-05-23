use anyhow::Result;
use clap::{Parser, Subcommand};
use std::time::{Duration, Instant};
use tokio;

mod helixdb;
mod neo4j;
mod types;

use crate::helixdb::HelixDBEngine;
use crate::neo4j::Neo4jEngine;
use crate::types::BenchmarkEngine;
use crate::types::{Benchmark, BenchmarkClient, Database};

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
        #[arg(short, long, default_value_t = 500_000)]
        count: usize,
        /// Database: helixdb, neo4j or others
        #[arg(short, long, default_value = "helixdb")]
        database: String,
        /// Endpoint URL (optional)
        #[arg(short, long)]
        endpoint: Option<String>,
    },
}

async fn run_benchmark(
    client: &mut dyn BenchmarkClient,
    operation: &str,
    count: usize,
) -> Result<(Duration, f64, f64)> {
    let start = Instant::now();
    match operation.to_lowercase().as_str() {
        "create" => client.create_records(count).await?,
        "read" => client.read_records().await?,
        "update" => client.update_records().await?,
        "delete" => client.delete_records().await?,
        "scan" => client.scan_records().await?,
        _ => return Err(anyhow::anyhow!("Unsupported operation: {}", operation)),

        /*
        "bulk_create" => {
            let pb = ProgressBar::new(count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Bulk Create",
                    )
                    .unwrap()
                    .progress_chars("##-"),
            );
            client
                .bulk_create_string(count, sample_value.clone())
                .await?;
            pb.finish_with_message("Bulk Create complete");
        }
        ("huge_traversal", KeyType::U32) => {
            let pb = ProgressBar::new(count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Huge Traversal",
                    )
                    .unwrap()
                    .progress_chars("##-"),
            );
            client.huge_traversal(count).await?;
            pb.finish_with_message("Huge Traversal complete");
        }
        */
    }

    let total_time = start.elapsed();
    let avg_time_per_request = total_time.as_secs_f64() / count as f64;
    let throughput = count as f64 / total_time.as_secs_f64();

    Ok((total_time, avg_time_per_request, throughput))
}

async fn run_all_benchmarks(
    client: &mut dyn BenchmarkClient,
    count: usize,
) -> Result<Vec<(String, Duration, f64, f64)>> {
    let mut results = Vec::new();

    let (create_duration, create_avg_time, create_throughput) =
        run_benchmark(client, "create", count).await?;
    results.push(("create".to_string(), create_duration, create_avg_time, create_throughput));

    let (read_duration, read_avg_time, read_throughput) =
        run_benchmark(client, "read", count).await?;
    results.push(("read".to_string(), read_duration, read_avg_time, read_throughput));

    let (update_duration, update_avg_time, update_throughput) =
        run_benchmark(client, "update", count).await?;
    results.push(("update".to_string(), update_duration, update_avg_time, update_throughput));

    //let (delete_duration, delete_avg_time, delete_throughput) =
    //    run_benchmark(client, "delete", count).await?;
    //results.push(("delete".to_string(), delete_duration, delete_avg_time, delete_throughput));

    let (scan_duration, scan_avg_time, scan_throughput) =
        run_benchmark(client, "scan", count).await?;
    results.push(("scan".to_string(), scan_duration, scan_avg_time, scan_throughput));

    /*
    let (bulk_create_duration, bulk_create_avg_time, bulk_create_throughput) =
        run_benchmark(client, "bulk_create", count, KeyType::U32).await?;
    results.push((
        "bulk_create".to_string(),
        bulk_create_duration,
        bulk_create_avg_time,
        bulk_create_throughput,
    ));

    let (huge_traversal_duration, huge_traversal_avg_time, huge_traversal_throughput) =
        run_benchmark(client, "huge_traversal", count, KeyType::U32).await?;
    results.push((
        "huge_traversal".to_string(),
        huge_traversal_duration,
        huge_traversal_avg_time,
        huge_traversal_throughput,
    ));
    */

    Ok(results)
}

fn database_name(database: Database) -> &'static str {
    match database {
        Database::HelixDB => "HelixDB",
        Database::Neo4j => "Neo4j",
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bench {
            operation,
            count,
            database,
            endpoint,
        } => {
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

            let mut client = engine.create_client().await?;

            if operation.to_lowercase() == "all" {
                let results = run_all_benchmarks(&mut *client, count).await?;
                println!(
                    "\nBenchmark Results for {} ({} operations):",
                    database_name(database),
                    count,
                );
                println!("{:-<80}", "");
                println!(
                    "{:<10} | {:<15} | {:<15} | {:<15}",
                    "Operation", "Total Time", "Avg Time/Req (ms)", "Throughput (ops/s)"
                );
                println!("{:-<80}", "");
                for (op, duration, avg_time, throughput) in results {
                    println!(
                        "{:<10} | {:<15} | {:<15.6} | {:<15.2}",
                        op,
                        format!("{:?}", duration),
                        avg_time * 1000.0,
                        throughput
                    );
                }
            } else {
                let (duration, avg_time, throughput) =
                    run_benchmark(&mut *client, &operation, count).await?;
                println!(
                    "Benchmark: {} {} operations on {}:\n\
                    Total Time: {:?}\n\
                    Avg Time/Request: {:.6} ms\n\
                    Throughput: {:.2} ops/s",
                    operation,
                    count,
                    database_name(database),
                    duration,
                    avg_time * 1000.0,
                    throughput
                );
            }
            // count exisiting records
            let count = client.count_records().await?;
            println!("Existing records: {}", count);
        }
    }

    Ok(())
}
