use crate::model::CreateOrderRequest;
use crate::signature::SignatureManager;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const BASE_URL: &str = "https://pro.edgex.exchange";

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Signature error: {0}")]
    SignatureError(#[from] crate::signature::SignatureError),
    #[error("API error: {0}")]
    ApiError(String),
}

pub struct EdgeXClient {
    client: Client,
    signature_manager: SignatureManager,
    base_url: String,
}

impl EdgeXClient {
    pub fn new(private_key: &str, base_url: Option<String>) -> Result<Self, ClientError> {
        let signature_manager = SignatureManager::new(private_key)?;
        let client = Client::builder().build()?;
        let base_url = base_url.unwrap_or_else(|| BASE_URL.to_string());

        Ok(Self {
            client,
            signature_manager,
            base_url,
        })
    }

    pub async fn create_order(&self, req: &CreateOrderRequest) -> Result<Value, ClientError> {
        let url = format!("{}/api/v1/private/order/createOrder", self.base_url);
        
        // TODO: The request object 'req' should already have l2Signature populated, 
        // OR we should sign it here.
        // For now, assuming caller or a builder helper handles signing before passing here, 
        // or we clone and sign here.
        
        // Let's assume we implement a helper to sign and create the request.
        // But for this raw method, we take the request as is.
        
        let body = serde_json::to_string(req).map_err(|e| ClientError::ApiError(e.to_string()))?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string();
        
        let path = "/api/v1/private/order/createOrder";
        // Header signature content usually: timestamp + method + path + body
        let _sign_payload = format!("{}{}{}{}", timestamp, "POST", path, body);
        
        // TODO: header signature implementation is uncertain.
        // If it requires L2 Key signing of this payload:
        // let header_signature = self.signature_manager.sign_message(&_sign_payload).await?;
        // But sign_message is not implemented for Stark key yet (needs definition of hash algo).
        // For now, use a placeholder or fail.
        // To proceed with SDK dev, we assume we can add this later.
        let header_signature = "0x0000000000000000000000000000000000000000".to_string(); // Temporary

        let mut headers = HeaderMap::new();
        headers.insert("X-edgeX-Api-Timestamp", HeaderValue::from_str(&timestamp).unwrap());
        headers.insert("X-edgeX-Api-Signature", HeaderValue::from_str(&header_signature).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let res = self.client.post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let text = res.text().await?;
            return Err(ClientError::ApiError(format!("Status: {}, Body: {}", status, text)));
        }

        let json: Value = res.json().await?;
        Ok(json)
    }

    pub async fn cancel_order(&self, req: &crate::model::CancelOrderRequest) -> Result<Value, ClientError> {
        let url = format!("{}/api/v1/private/order/cancelOrderById", self.base_url);
        // Uses same Header auth mechanism
        
        let body = serde_json::to_string(req).map_err(|e| ClientError::ApiError(e.to_string()))?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string();
        let path = "/api/v1/private/order/cancelOrderById";
        
        let header_signature = "0x0000000000000000000000000000000000000000".to_string(); // Temporary

        let mut headers = HeaderMap::new();
        headers.insert("X-edgeX-Api-Timestamp", HeaderValue::from_str(&timestamp).unwrap());
        headers.insert("X-edgeX-Api-Signature", HeaderValue::from_str(&header_signature).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let res = self.client.post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let text = res.text().await?;
            return Err(ClientError::ApiError(format!("Status: {}, Body: {}", status, text)));
        }

        let json: Value = res.json().await?;
        Ok(json)
    }

    pub async fn get_open_orders(&self, account_id: u64) -> Result<Vec<crate::model::OpenOrder>, ClientError> {
        let url = format!("{}/api/v1/private/order/getOpenOrders", self.base_url);
        let params = [("accountId", account_id.to_string())];
        
        // GET request with query params
        // Header signature usually requires Path + QueryString? 
        // Or strictly Request Body?
        // Docs usually specify. For now assuming timestamp+method+path+query OR just path.
        // If GET, body is empty.
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string();
        let header_signature = "0x0000000000000000000000000000000000000000".to_string(); // Temporary

        let mut headers = HeaderMap::new();
        headers.insert("X-edgeX-Api-Timestamp", HeaderValue::from_str(&timestamp).unwrap());
        headers.insert("X-edgeX-Api-Signature", HeaderValue::from_str(&header_signature).unwrap());

        let res = self.client.get(&url)
            .headers(headers)
            .query(&params)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let text = res.text().await?;
            return Err(ClientError::ApiError(format!("Status: {}, Body: {}", status, text)));
        }

        // Response structure might be { "code": "...", "data": [...] }
        // We'll parse Value first then generic.
        let json: Value = res.json().await?;
        // Assuming "data" field contains list, or root is list.
        // Need to check docs for response format.
        // Usually "data": [ ... ]
        if let Some(data) = json.get("data") {
             let orders: Vec<crate::model::OpenOrder> = serde_json::from_value(data.clone()).map_err(|e| ClientError::ApiError(e.to_string()))?;
             Ok(orders)
        } else {
             // Fallback if root is array
             let orders: Vec<crate::model::OpenOrder> = serde_json::from_value(json).map_err(|e| ClientError::ApiError(e.to_string()))?;
             Ok(orders)
        }
    }

    pub async fn get_fills(&self, account_id: u64) -> Result<Vec<crate::model::Fill>, ClientError> {
        let url = format!("{}/api/v1/private/order/getFills", self.base_url);
        let params = [("accountId", account_id.to_string())];
        
        // Similar GET auth pattern
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string();
        let header_signature = "0x0000000000000000000000000000000000000000".to_string(); // Temporary

        let mut headers = HeaderMap::new();
        headers.insert("X-edgeX-Api-Timestamp", HeaderValue::from_str(&timestamp).unwrap());
        headers.insert("X-edgeX-Api-Signature", HeaderValue::from_str(&header_signature).unwrap());

        let res = self.client.get(&url)
            .headers(headers)
            .query(&params)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            let text = res.text().await?;
            return Err(ClientError::ApiError(format!("Status: {}, Body: {}", status, text)));
        }

        let json: Value = res.json().await?;
        if let Some(data) = json.get("data") {
             let fills: Vec<crate::model::Fill> = serde_json::from_value(data.clone()).map_err(|e| ClientError::ApiError(e.to_string()))?;
             Ok(fills)
        } else {
             let fills: Vec<crate::model::Fill> = serde_json::from_value(json).map_err(|e| ClientError::ApiError(e.to_string()))?;
             Ok(fills)
        }
    }
}
