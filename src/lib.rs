extern crate hmac;
extern crate reqwest;
extern crate serde_json;
extern crate sha2;
extern crate uuid;

use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;
use hmac::{Hmac, Mac};
use reqwest::Method;
use sha2::Sha256;
use serde::Deserialize;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

pub struct V1 {
    base_url: String,
    key: String,
    secret: String,
    client: reqwest::Client,
}

impl V1 {
    pub fn new(base_url: &str, key: &str, secret: &str) -> V1 {
        V1 {
            base_url: base_url.to_string(),
            key: key.to_string(),
            secret: secret.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn get_account_id(&self) -> Result<String, reqwest::Error> {
        let mut response = self.get(Method::GET, "/api/v1/account/id")?;
        response.text().map(|s| s.trim_matches('"').to_string())
    }

    pub fn get_account_balance(&self) -> Result<Vec<Balance>, reqwest::Error> {
        let mut response = self.get(Method::GET, "/api/v1/account/balances")?;
        response.text().map(|json| serde_json::from_str(&json).unwrap())
    }

    pub fn get_orderbook(&self, pair: &str) -> Result<Orderbook, reqwest::Error> {
        let mut response = self.client.get(&format!("{}/api/v1/public/market/orderbook/{}", self.base_url, pair)).send()?;
        response.text().map(|json| serde_json::from_str(&json).unwrap())
    }

    pub fn get_market_order(&self, id: &str) -> Result<Order, reqwest::Error> {
        let mut response = self.get(Method::GET, &format!("/api/v1/market/orders/{}", id))?;
        response.text().map(|json| serde_json::from_str(&json).unwrap())
    }

    pub fn get_market_orders(&self) -> Result<Vec<Order>, reqwest::Error> {
        let mut response = self.get(Method::GET, "/api/v1/market/orders")?;
        response.text().map(|json| serde_json::from_str(&json).unwrap())
    }

    pub fn new_market_order(&self, order: &MakeOrderRequest) -> Result<MakeOrderResponse, reqwest::Error> {
        let url = format!("/api/v1/market/orders?pair={}&price={}&buySell={}&volume={}&volumeCurrency={}&otherCurrency={}&submitId={}",
            order.pair, order.price, order.buy_sell, order.volume, order.volume_currency, order.other_currency, order.submit_id);
        let mut response = self.get(Method::POST, &url)?;
        response.text().map(|json| serde_json::from_str(&json).unwrap())
    }

    pub fn close_market_order(&self, id: &str) -> Result<Order, reqwest::Error> {
        let mut response = self.get(Method::POST, &format!("/api/v1/market/orders/close/{}", id))?;
        response.text().map(|json| serde_json::from_str(&json).unwrap())
    }

    fn get(&self, method: Method, uri: &str) -> Result<reqwest::Response, reqwest::Error> {
        let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string();
        let msg = format!("{}{}", uri, ts);
        let mut mac = HmacSha256::new_varkey(self.secret.as_bytes())
            .expect("Invalid key length");
        mac.input(msg.as_bytes());
        let signature = to_hex_string(mac.result().code().as_slice());
        self.client.request(method, &format!("{}{}", self.base_url, uri))
            .header("X-API-KEY", self.key.clone())
            .header("X-API-NONCE", ts)
            .header("X-API-SIGNATURE", signature)
            .send()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Balance {
    pub currency: String,
    #[serde(rename = "balanceAll")]
    pub balance_all: String,
    #[serde(rename = "balanceAvailable")]
    pub balance_available: String,
    #[serde(rename = "balanceReserved")]
    pub balance_reserved: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Orderbook {
    pub pair: String,
    pub bids: Vec<OrderbookEntry>,
    pub asks: Vec<OrderbookEntry>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct OrderbookEntry {
    pub price: String,
    #[serde(rename = "baseVolume")]
    pub base_volume: String,
    #[serde(rename = "marketVolume")]
    pub market_volume: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Order {
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "submitId")]
    pub submit_id: String,
    #[serde(rename = "submitTs")]
    pub submit_ts: String,
    #[serde(rename = "updateTs")]
    pub update_ts: String,
    pub status: String,
    pub market: String,
    #[serde(rename = "buySell")]
    pub buy_sell: String,
    pub volume: String,
    #[serde(rename = "volumeCurrency")]
    pub volume_currency: String,
    #[serde(rename = "otherCurrency")]
    pub other_currency: String,
    pub price: String,
    pub completion: String,
    #[serde(rename = "soldAmount")]
    pub sold_amount: String,
    #[serde(rename = "boughtAmount")]
    pub bought_amount: String,
    #[serde(rename = "feeRate")]
    pub fee_rate: String,
    #[serde(rename = "feeAmountMax")]
    pub fee_amount_max: String,
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Order {} {} {} {}", self.order_id, self.status, self.market, self.buy_sell)
    }
}

pub struct MakeOrderRequest<'a> {
    submit_id: &'a str,
    pair: &'a str,
    price: &'a str,
    buy_sell: &'a str,
    volume: &'a str,
    volume_currency: &'a str,
    other_currency: &'a str,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct MakeOrderResponse {
    duplicate: Option<bool>,
    #[serde(rename = "orderId")]
    order_d: Option<String>,
    message: Option<String>,
    errors: Option<HashMap<String, Vec<String>>>,
}

fn to_hex_string(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02x}", b))
        .fold(String::new(), |s1, s2| s1 + &s2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_rate() {
        let key = "key";
        let secret = "secret";
        let wt = V1::new("https:/api.walutomat.pl", key, secret);
        let ret = wt.new_market_order(&MakeOrderRequest {
            buy_sell: "SELL",
            pair: "EUR_PLN",
            price: "4.0000",
            volume: "1.00",
            volume_currency: "PLN",
            other_currency: "EUR",
            submit_id: &Uuid::new_v4().to_string(),
        });
        let ret = wt.close_market_order("123");
        match ret {
            Ok(order) => println!("{}", order),
            err => println!("{:?}", err)
        }
        
        //let ret = wt.get_orderbook("EUR_PLN").unwrap();
        //assert_eq!(ret, Orderbook {pair: "".to_string(), bids: vec![], asks: vec![] });
        //let ret = wt.get_account_id().unwrap();
        //let ret = wt.get_account_balance().unwrap();
    }
}
