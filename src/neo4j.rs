use std::collections::HashMap;

use crate::types::{Benchmark, BenchmarkClient, BenchmarkEngine, Projection, Scan};
use crate::utils::extract_string_field;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

pub struct Neo4jClient {
    endpoint: String,
    client: Client,
}

impl Neo4jClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: Client::new(),
        }
    }

    async fn execute_cypher(&self, query: &str, params: Value) -> Result<Value> {
        let url = format!("{}/db/neo4j/tx/commit", self.endpoint);
        let body = json!({
            "statements": [{"statement": query, "parameters": params}]
        });
        let response = self
            .client
            .post(&url)
            .json(&body)
            .basic_auth("neo4j", Some("neo4jtest"))
            .send()
            .await?;
        if response.status().is_success() {
            response.json::<Value>().await.map_err(Into::into)
        } else {
            Err(anyhow::anyhow!("Request failed: {}", response.status()))
        }
    }
}

#[async_trait]
impl BenchmarkClient for Neo4jClient {
    async fn startup(&self) -> Result<()> {
        self.execute_cypher("RETURN 1", json!({})).await?;
        Ok(())
    }

    async fn create_u32(&self, key: u32, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let query = "CREATE (n:Record {id: $id, data: $data})";
        let params = json!({"id": key.to_string(), "data": data});
        self.execute_cypher(query, params).await?;
        Ok(())
    }

    async fn create_string(&self, key: String, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let query = "CREATE (n:Record {id: $id, data: $data})";
        let params = json!({"id": key, "data": data});
        self.execute_cypher(query, params).await?;
        Ok(())
    }

    async fn read_u32(&self, key: u32) -> Result<()> {
        let query = "MATCH (n:Record {id: $id}) RETURN n";
        let params = json!({"id": key.to_string()});
        self.execute_cypher(query, params).await?;
        Ok(())
    }

    async fn read_string(&self, key: String) -> Result<()> {
        let query = "MATCH (n:Record {id: $id}) RETURN n";
        let params = json!({"id": key});
        self.execute_cypher(query, params).await?;
        Ok(())
    }

    async fn update_u32(&self, key: u32, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let query = "MATCH (n:Record {id: $id}) SET n.data = $data";
        let params = json!({"id": key.to_string(), "data": data});
        self.execute_cypher(query, params).await?;
        Ok(())
    }

    async fn update_string(&self, key: String, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let query = "MATCH (n:Record {id: $id}) SET n.data = $data";
        let params = json!({"id": key, "data": data});
        self.execute_cypher(query, params).await?;
        Ok(())
    }

    async fn delete_u32(&self, key: u32) -> Result<()> {
        let query = "MATCH (n:Record {id: $id}) DELETE n";
        let params = json!({"id": key.to_string()});
        self.execute_cypher(query, params).await?;
        Ok(())
    }

    async fn delete_string(&self, key: String) -> Result<()> {
        let query = "MATCH (n:Record {id: $id}) DELETE n";
        let params = json!({"id": key});
        self.execute_cypher(query, params).await?;
        Ok(())
    }

    async fn scan_u32(&self, scan: &Scan) -> Result<usize> {
        self.scan(scan).await
    }

    async fn scan_string(&self, scan: &Scan) -> Result<usize> {
        self.scan(scan).await
    }
}

impl Neo4jClient {
    async fn scan(&self, scan: &Scan) -> Result<usize> {
        let limit = scan.limit.unwrap_or(100);
        let offset = scan.start.unwrap_or(0);
        let (query, params) = match scan.projection()? {
            Projection::Count => ("MATCH (n:Record) RETURN count(n)", json!({})),
            _ => (
                "MATCH (n:Record) RETURN n LIMIT $limit SKIP $offset",
                json!({"limit": limit, "offset": offset}),
            ),
        };
        let response = self.execute_cypher(query, params).await?;
        match scan.projection()? {
            Projection::Count => {
                let count = response["results"][0]["data"][0]["row"][0]
                    .as_u64()
                    .unwrap_or(0) as usize;
                Ok(count)
            }
            _ => {
                let rows = response["results"][0]["data"]
                    .as_array()
                    .map(|arr| arr.len())
                    .unwrap_or(0);
                Ok(rows)
            }
        }
    }
}

// Engine for Neo4j
pub struct Neo4jEngine {
    endpoint: String,
}

#[async_trait]
impl BenchmarkEngine for Neo4jEngine {
    async fn setup(options: &Benchmark) -> Result<Self> {
        let endpoint = options
            .endpoint
            .as_deref()
            .unwrap_or("http://localhost:7474")
            .to_string();
        Ok(Self { endpoint })
    }

    async fn create_client(&self) -> Result<Box<dyn BenchmarkClient>> {
        let client = Neo4jClient::new(self.endpoint.clone());
        client.startup().await?;
        Ok(Box::new(client))
    }
}
