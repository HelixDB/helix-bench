use crate::types::{Benchmark, BenchmarkClient, BenchmarkEngine, Projection, Scan};
use crate::utils::extract_string_field;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{Value, json};
use uuid::Uuid;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct Neo4jClient {
    endpoint: String,
    client: Client,
    ids: Vec<Uuid>,
}

impl Neo4jClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: Client::new(),
            ids: Vec::new(),
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

    async fn create_records(&mut self, count: usize) -> Result<()> {
        let pb = ProgressBar::new(count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) Create")
                .unwrap()
                .progress_chars("##-"),
        );
        self.ids.extend((0..count).map(|_| Uuid::new_v4()));
        let query = "CREATE (n:Record {id: $id, data: $data})";
        for k in self.ids.clone().into_iter() {
            let params = json!({"id": k.to_string(), "data": "test_value"});
            self.execute_cypher(query, params).await?;
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
        let query = "MATCH (n:Record {id: $id}) RETURN n";
        for k in self.ids.clone().into_iter() {
            let params = json!({"id": k.to_string()});
            self.execute_cypher(query, params).await?;
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
            let query = "MATCH (n:Record {id: $id}) SET n.data = $data";
            let params = json!({"id": k.to_string(), "data": "updated_value"});
            self.execute_cypher(query, params).await?;
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
        let query = "MATCH (n:Record {id: $id}) DELETE n";
        for k in self.ids.clone().into_iter() {
            let params = json!({"id": k.to_string()});
            self.execute_cypher(query, params).await?;
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
        let query = "MATCH (n) RETURN count(n) as count";
        let params = json!({});
        let response = self.execute_cypher(query, params).await?;
        println!("Count records result: {:?}", response);
        Ok(response["results"][0]["data"][0]["row"][0]
            .as_u64()
            .unwrap_or(0) as usize)
    }

    /*
    async fn bulk_create_string(&self, count: usize, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let query = "
            UNWIND range(0, $count - 1) as i
            CREATE (n:Record {name: i, data: $data})
            WITH n, i
            MATCH (other:Record)
            WHERE other <> n AND id(other) >= 0 AND id(other) < i
            WITH n, collect(other) as others
            UNWIND range(1,3) as _
            WITH n, others, toInteger(rand() * size(others)) as idx
            WHERE idx < size(others)
            MATCH (other) WHERE other = others[idx]
            CREATE (n)-[:KNOWS]->(other)";
        let params = json!({"count": count, "data": data});
        self.execute_cypher(query, params).await?;
        Ok(())
    }

    async fn huge_traversal(&self, count: usize) -> Result<()> {
        let query = r#"
        MATCH (start:Record)
        MATCH path = (start)-[:KNOWS]->()-[:KNOWS]->(n2:Record)
        WHERE n2.name < 1000
        MATCH (n2)-[:KNOWS]->()-[:KNOWS]->()-[:KNOWS]->()-[:KNOWS]->(n6)
        WITH DISTINCT n6 as result
        RETURN result
        LIMIT 100
        "#;
        let params = json!({});  // count parameter not needed
        let response = self.execute_cypher(query, params).await?;
        println!("Huge traversal result: {:?}", response);
        Ok(())
    }
    */
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
