use crate::{
    types::{Benchmark, BenchmarkClient, BenchmarkEngine, Projection, Scan},
    utils::*,
};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

struct HelixDBClient {
    endpoint: String,
    client: Client,
    ids: Vec<Uuid>,
}

impl HelixDBClient {
    fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: Client::new(),
            ids: Vec::new(),
        }
    }

    async fn make_request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        let url = format!("{}{}", self.endpoint, path);
        let request = match method {
            "POST" => self.client.post(&url),
            _ => unreachable!(),
        };
        let request = if let Some(body) = body {
            request.json(&body)
        } else {
            request
        };
        let response = request.send().await.map_err(
            |e| {
                println!("Request failed: {}", e);
                anyhow::anyhow!("Request failed: {}", e)
            }
        )?;
        if response.status().is_success() {
            response.json::<Value>().await.map_err(Into::into)
        } else {
            Err(anyhow::anyhow!("Request failed: {}", response.status()))
        }
    }
}

#[async_trait]
impl BenchmarkClient for HelixDBClient {
    async fn startup(&self) -> Result<()> {
        Ok(()) // no specific startup needed; assume server is running
    }

    async fn create_records(&mut self, count: usize) -> Result<()> {
        let pb = ProgressBar::new(count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Create")
                .unwrap()
                .progress_chars("##-"),
        );
        for _ in 0..count {
            let res = self
                .make_request("POST", "/create_record", Some(json!({"data": "test_value"})))
                .await?;
            self.ids.push(
                res["record"][0]["id"]
                .as_str()
                .expect("ID is not a string")
                .parse::<Uuid>()
                .expect("Failed to parse UUID")
            );
            pb.inc(1);
        }
        pb.finish_with_message("Create complete");
        Ok(())
    }

    async fn read_records(&self) -> Result<()> {
        let pb = ProgressBar::new(self.ids.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Read")
                .unwrap()
                .progress_chars("##-"),
        );
        for k in self.ids.clone().into_iter() {
            let body = json!({"id": k.to_string()});
            let res = self.make_request("POST", "/read_record", Some(body))
                .await?;
            assert!(res["record"][0]["data"] == "test_value", "data is correct");
            pb.inc(1);
        }
        pb.finish_with_message("Read complete");
        Ok(())
    }

    async fn update_records(&self) -> Result<()> {
        let pb = ProgressBar::new(self.ids.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Update")
                .unwrap()
                .progress_chars("##-"),
        );
        for k in self.ids.clone().into_iter() {
            let body = json!({"id": k.to_string(), "data": "updated_value"});
            self.make_request("POST", "/update_record", Some(body))
                .await?;
            pb.inc(1);
        }
        pb.finish_with_message("Update complete");
        Ok(())
    }

    async fn delete_records(&self) -> Result<()> {
        let pb = ProgressBar::new(self.ids.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Delete")
                .unwrap()
                .progress_chars("##-"),
        );
        for k in self.ids.clone().into_iter() {
            let body = json!({"id": k.to_string()});
            self.make_request("POST", "/delete_record", Some(body))
                .await?;
            pb.inc(1);
        }
        pb.finish_with_message("Delete complete");
        Ok(())
    }

    async fn scan_records(&self) -> Result<()> {
        let count = self.ids.len();
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] Running scan...")
                .unwrap(),
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        let scan = Scan::new(Some(count), None, Projection::Full);
        let _ = self.scan(&scan).await;
        pb.finish_with_message("Scan complete");
        Ok(())
    }

    async fn count_records(&self) -> Result<usize> {
        let res = self
            .make_request("POST", "/count_records", None)
            .await?;
        println!("res: {:?}", res);
        Ok(0)
        //let count = res.get("count").unwrap();
        //Ok(count.as_u64().unwrap_or(0) as usize)
    }

    async fn create_vectors(&self, count: usize) -> Result<()> {
        let pb = ProgressBar::new(count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Create vectors")
                .unwrap()
                .progress_chars("##-"),
        );
        let rnd_vectors = generate_random_vectors(count, 768);
        for vec in rnd_vectors {
            let _ = self
                .make_request("POST", "/create_vector", Some(json!({"vec": vec})))
                .await?;
            pb.inc(1);
        }
        pb.finish_with_message("Create complete");
        Ok(())
    }

    async fn search_vectors(&self, count: usize) -> Result<()> {
        let pb = ProgressBar::new(count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Search vectors")
                .unwrap()
                .progress_chars("##-"),
        );
        let rnd_vectors = generate_random_vectors(count, 768);
        for vec in rnd_vectors {
            let _ = self
                .make_request("POST", "/search_vector", Some(json!({"query": vec, "k": 7})))
                .await?;
            pb.inc(1);
        }
        pb.finish_with_message("Create complete");
        Ok(())
    }

    /*
    async fn bulk_create(&self, count: usize) -> Result<()> {
        let body = json!({"count": count, "data": "test_value"});
        let res = self
            .make_request("POST", "/insert", Some(body))
            .await?;
        println!("Bulk create result: {:?}", res);
        Ok(())
    }

    async fn huge_traversal(&self, count: usize) -> Result<()> {
        let body = json!({"count": count});
        let res = self
            .make_request("POST", "/huge_traversal", Some(body))
            .await?;
        println!("Huge traversal result: {:?}", res);
        Ok(())
    }
    */
}

impl HelixDBClient {
    async fn scan(&self, scan: &Scan) -> Result<usize> {
        let limit = scan.limit.unwrap_or(100) as i64;
        let offset = scan.start.unwrap_or(0) as i64;
        match scan.projection()? {
            Projection::Id | Projection::Full => {
                let body = json!({"limit": limit, "offset": offset});
                let response = self
                    .make_request("POST", "/scan_records", Some(body))
                    .await?;
                let count = response.as_array().map(|arr| arr.len()).unwrap_or(0);
                Ok(count)
            }
            Projection::Count => {
                let response = self.make_request("POST", "/count_records", None).await?;
                let count = response.as_i64().unwrap_or(0) as usize;
                Ok(count)
            }
        }
    }
}

pub struct HelixDBEngine {
    endpoint: String,
}

#[async_trait]
impl BenchmarkEngine for HelixDBEngine {
    async fn setup(options: &Benchmark) -> Result<Self> {
        let endpoint = options
            .endpoint
            .as_deref()
            .unwrap_or("http://localhost:6969")
            .to_string();
        Ok(Self { endpoint })
    }

    async fn create_client(&self) -> Result<Box<dyn BenchmarkClient>> {
        let client = HelixDBClient::new(self.endpoint.clone());
        client.startup().await?;
        Ok(Box::new(client))
    }
}
