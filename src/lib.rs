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
use serde::de::DeserializeOwned;

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

    pub fn get_account_id(&self) -> Result<String, Error> {
        self.request(Method::GET, "/api/v1/account/id")
    }

    pub fn get_account_balance(&self) -> Result<Vec<Balance>, Error> {
        self.request(Method::GET, "/api/v1/account/balances")
    }

    pub fn get_orderbook(&self, pair: &str) -> Result<Orderbook, Error> {
        let mut response = self.client.get(&format!("{}/api/v1/public/market/orderbook/{}", self.base_url, pair)).send()?;
        let json: String = response.text()?;
        serde_json::from_str(&json)
            .map_err(|err| Error::from(err))
    }

    pub fn get_market_order(&self, id: &str) -> Result<Order, Error> {
        self.request(Method::GET, &format!("/api/v1/market/orders/{}", id))
    }

    pub fn get_market_orders(&self) -> Result<Order, Error> {
        self.request(Method::GET, "/api/v1/market/orders")
    }

    pub fn new_market_order(&self, order: &MakeOrderRequest) -> Result<MakeOrderResponse, Error> {
        let url = format!("/api/v1/market/orders?pair={}&price={}&buySell={}&volume={}&volumeCurrency={}&otherCurrency={}&submitId={}",
            order.pair, order.price, order.buy_sell, order.volume, order.volume_currency, order.other_currency, order.submit_id);
        self.request(Method::POST, &url)
    }

    pub fn close_market_order(&self, id: &str) -> Result<Order, Error> {
        self.request(Method::POST, &format!("/api/v1/market/orders/close/{}", id))
    }

    fn request<T>(&self, method: Method, uri: &str) -> Result<T, Error>
    where T: DeserializeOwned {
        let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string();
        let mut response = self.client.request(method, &format!("{}{}", self.base_url, uri))
            .header("X-API-KEY", self.key.clone())
            .header("X-API-NONCE", ts.clone())
            .header("X-API-SIGNATURE", signature(uri, &ts, &self.secret))
            .send()?;
        let json: String = response.text()?;
        serde_json::from_str(&json)
            .map_err(|err| Error::from(err))
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

#[derive(Clone, Debug, PartialEq, Deserialize)]
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

#[derive(Debug)]
pub enum Error {
    SerdeErr(serde_json::Error),
    ReqwestErr(reqwest::Error),
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeErr(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::ReqwestErr(error)
    }
}

fn signature(uri: &str, ts: &str, secret: &str) -> String {
    let msg = format!("{}{}", uri, ts);
    let mut mac = HmacSha256::new_varkey(secret.as_bytes())
        .expect("Invalid key length");
    mac.input(msg.as_bytes());
    to_hex_string(mac.result().code().as_slice())
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
    fn signing() {
        // compare with example at
        // https://api.walutomat.pl/#us%C5%82ugi-wymagaj%C4%85ce-uwierzytelnienia-kluczem-wsp%C3%B3%C5%82dzielonym
        let secret = "766j0m0hcaz0ml8erklf0ww18";
        let uri = "/api/v1/market/orders/close/5137bdb7-acde-41ff-aeb2-0908af0bd3d9";
        let nonce = "1517480182188";
        let s = signature(uri, nonce, secret);
        assert_eq!(s, "b789acef01059fbf40b787be6ce8ea414a0130106a9dd5eb57c40fd2ea4d80a");
    }
}
