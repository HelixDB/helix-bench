use std::collections::HashMap;

use crate::types::{Benchmark, BenchmarkClient, BenchmarkEngine, Projection, Scan};
use crate::utils::extract_string_field;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Number, Value};

struct HelixDBClient {
    endpoint: String,
    client: Client,
    id_mapping: HashMap<u32, String>,
}

impl HelixDBClient {
    fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: Client::new(),
            id_mapping: HashMap::new(),
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
        // No specific startup needed; assume server is running
        Ok(())
    }

    async fn create_u32(&self, key: u32, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let body = json!({"id": key.to_string(), "data": data});
        let res = self
            .make_request("POST", "/create_record", Some(body))
            .await?;
        Ok(())
    }

    async fn create_string(&self, key: String, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let body = json!({"id": key, "data": data});
        let res = self
            .make_request("POST", "/create_record", Some(body))
            .await?;
        Ok(())
    }

    async fn bulk_create_string(&self, count: usize, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let body = json!({"count": count, "data": data});
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

    async fn read_u32(&self, key: u32) -> Result<()> {
        let body = json!({"id": key.to_string()});
        let res = self
            .make_request("POST", "/read_record", Some(body))
            .await?;
        Ok(())
    }

    async fn read_string(&self, key: String) -> Result<()> {
        let body = json!({"id": key});
        self.make_request("POST", "/read_record", Some(body))
            .await?;
        Ok(())
    }

    async fn update_u32(&self, key: u32, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let body = json!({"id": key.to_string(), "data": data});
        self.make_request("POST", "/update_record", Some(body))
            .await?;
        Ok(())
    }

    async fn update_string(&self, key: String, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let body = json!({"id": key, "data": data});
        self.make_request("POST", "/update_record", Some(body))
            .await?;
        Ok(())
    }

    async fn delete_u32(&self, key: u32) -> Result<()> {
        let body = json!({"id": key.to_string()});
        self.make_request("POST", "/delete_record", Some(body))
            .await?;
        Ok(())
    }

    async fn delete_string(&self, key: String) -> Result<()> {
        let body = json!({"id": key});
        self.make_request("POST", "/delete_record", Some(body))
            .await?;
        Ok(())
    }

    async fn scan_u32(&self, scan: &Scan) -> Result<usize> {
        self.scan(scan).await
    }

    async fn scan_string(&self, scan: &Scan) -> Result<usize> {
        self.scan(scan).await
    }

    async fn count_records(&self) -> Result<usize> {
        let res = self
            .make_request("POST", "/count_records", None)
            .await?;
        // get count field from res
        let count = res.get("count").unwrap();
        Ok(count.as_u64().unwrap_or(0) as usize)
    }
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
