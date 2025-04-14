

struct HelixDBClient {
    endpoint: String,
    client: Client,
}

impl HelixDBClient {
    fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: Client::new(),
        }
    }

    async fn make_request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        let url = format!("{}{}", self.endpoint, path);
        let request = match method {
            "POST" => self.client.post(&url),
            "GET" => self.client.get(&url),
            _ => unreachable!(),
        };
        let request = if let Some(body) = body {
            request.json(&body)
        } else {
            request
        };
        let response = request.send().await?;
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
        self.make_request("POST", "/create_record", Some(body)).await?;
        Ok(())
    }

    async fn create_string(&self, key: String, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let body = json!({"id": key, "data": data});
        self.make_request("POST", "/create_record", Some(body)).await?;
        Ok(())
    }

    async fn read_u32(&self, key: u32) -> Result<()> {
        let body = json!({"id": key.to_string()});
        self.make_request("POST", "/read_record", Some(body)).await?;
        Ok(())
    }

    async fn read_string(&self, key: String) -> Result<()> {
        let body = json!({"id": key});
        self.make_request("POST", "/read_record", Some(body)).await?;
        Ok(())
    }

    async fn update_u32(&self, key: u32, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let body = json!({"id": key.to_string(), "data": data});
        self.make_request("POST", "/update_record", Some(body)).await?;
        Ok(())
    }

    async fn update_string(&self, key: String, val: Value) -> Result<()> {
        let data = extract_string_field(&val)?;
        let body = json!({"id": key, "data": data});
        self.make_request("POST", "/update_record", Some(body)).await?;
        Ok(())
    }

    async fn delete_u32(&self, key: u32) -> Result<()> {
        let body = json!({"id": key.to_string()});
        self.make_request("POST", "/delete_record", Some(body)).await?;
        Ok(())
    }

    async fn delete_string(&self, key: String) -> Result<()> {
        let body = json!({"id": key});
        self.make_request("POST", "/delete_record", Some(body)).await?;
        Ok(())
    }

    async fn scan_u32(&self, scan: &Scan) -> Result<usize> {
        self.scan(scan).await
    }

    async fn scan_string(&self, scan: &Scan) -> Result<usize> {
        self.scan(scan).await
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
                let count = response
                    .as_array()
                    .map(|arr| arr.len())
                    .unwrap_or(0);
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

// Engine for HelixDB
struct HelixDBEngine {
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

