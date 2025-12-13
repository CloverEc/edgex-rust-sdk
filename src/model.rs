use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    pub price: String,
    pub size: String,
    pub r#type: OrderType,
    pub time_in_force: TimeInForce,
    pub account_id: u64,
    pub contract_id: u64,
    pub side: OrderSide,
    // L2 Auth fields
    pub l2_nonce: u64,
    pub l2_value: String,
    pub l2_size: String,
    pub l2_limit_fee: String,
    pub l2_expire_time: u64,
    pub l2_signature: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderRequest {
    pub account_id: u64,
    pub order_id: Option<u64>,
    pub client_order_id: Option<String>,
    pub contract_id: u64,
    // L2 Auth fields
    pub l2_nonce: u64,
    pub l2_signature: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub order_id: u64,
    pub client_order_id: Option<String>,
    pub status: String,
    // Add other fields as discovered from API responses
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrder {
    pub order_id: u64,
    pub contract_id: u64,
    pub price: String,
    pub size: String,
    pub side: OrderSide,
    pub status: String,
    pub filled_size: String,
    pub remaining_size: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
    pub id: u64,
    pub order_id: u64,
    pub contract_id: u64,
    pub price: String,
    pub size: String,
    pub side: OrderSide,
    pub time: u64,
    pub fee: String,
    pub fee_asset_id: u64,
}
