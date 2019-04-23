extern crate reqwest;
extern crate serde_json;
extern crate uuid;

use std::collections::HashMap;
use std::fmt;
use serde::Deserialize;
use serde::de::{Deserializer, DeserializeOwned};

use super::Error;

pub struct API {
    base_url: String,
    key: String,
    client: reqwest::Client,
}

impl API {
    pub fn new(base_url: &str, key: &str) -> API {
        API {
            base_url: base_url.to_string(),
            key: key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn account_balance(&self) -> Result<ResultResponse<Vec<Balance>>, Error> {
        self.get("/api/v2.0.0/account/balances")
    }

    pub fn direct_fx_rate(&self, pair: &str) -> Result<ResultResponse<DirectFxRate>, Error> {
        self.get(&format!("/api/v2.0.0/direct_fx/rates?currency_pair={}", pair))
    }

    pub fn direct_fx_exchange(&self, order: &DirectOrderRequest) -> Result<ResultResponse<DirectFxExchange>, Error> {
        let body = format!("dryRun={}submitId={}&currencyPair={}&buySell={}&volume={}&volumeCurrency={}&ts={}",
            order.dry_run, order.submit_id, order.currency_pair, order.buy_sell, order.volume, order.volume_currency, order.ts);
        self.post("/api/v2.0.0/direct_fx/exchanges", &body)
    }

    pub fn market_fx_best_offers(&self, pair: &str) -> Result<ResultResponse<Orderbook>, Error> {
        let mut response = self.client.get(&format!("{}/api/v2.0.0/market_fx/best_offers?currencyPair={}", self.base_url, pair)).send()?;
        let json: String = response.text()?;
        serde_json::from_str(&json)
            .map_err(|err| Error::from(err))
    }

    pub fn market_fx_orders(&self, order_id: Option<&str>) -> Result<ResultResponse<Order>, Error> {
        let query: String = order_id.map_or("".to_string(), |order_id| format!("?orderId={}", order_id));
        self.get(&format!("/api/v2.0.0/market_fx/orders{}", &query))
    }

    pub fn market_fx_order(&self, order: &MarketOrderRequest) -> Result<ResultResponse<MakeOrderResponse>, Error> {
        let body = format!("dryRun={}submitId={}&currencyPair={}&buySell={}&volume={}&volumeCurrency={}&limitPrice={}",
            order.dry_run, order.submit_id, order.currency_pair, order.buy_sell, order.volume, order.volume_currency, order.limit_price);
        self.post("/api/v2.0.0/market_fx/orders", &body)
    }

    pub fn market_fx_order_close(&self, id: &str) -> Result<ResultResponse<Order>, Error> {
        self.post("/api/v2.0.0/market_fx/orders/close", &format!("orderId={}", id))
    }

    pub fn payout(&self, payout: &PayoutRequest) -> Result<ResultResponse<PayoutResponse>, Error> {
        unimplemented!();
    }

    fn post<T>(&self, uri: &str, body: &str) -> Result<T, Error>
    where T: DeserializeOwned {
        let mut response = self.client.post(&format!("{}{}", self.base_url, uri))
            .header("X-API-KEY", self.key.clone())
            .body(body.to_string())
            .send()?;
        let json: String = response.text()?;
        serde_json::from_str(&json)
            .map_err(|err| Error::from(err))
    }

    fn get<T>(&self, uri: &str) -> Result<T, Error>
    where T: DeserializeOwned {
        let mut response = self.client.get(&format!("{}{}", self.base_url, uri))
            .header("X-API-KEY", self.key.clone())
            .send()?;
        let json: String = response.text()?;
        serde_json::from_str(&json)
            .map_err(|err| Error::from(err))
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct ResultResponse<T> {
    pub success: bool,
    pub result: Option<T>,
    pub errors: Option<Vec<ErrorType>>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct ErrorType {
    pub key: String,
    pub description: String,
    #[serde(rename = "errorData")]
    pub error_data: Vec<KeyValue>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct DirectFxExchange {
    #[serde(rename = "exchangeId")]
    pub exchange_id: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct DirectFxRate {
    pub ts: String,
    #[serde(rename = "currencyPair")]
    pub currency_pair: String,
    #[serde(rename = "buyRate")]
    pub buy_rate: String,
    #[serde(rename = "sellRate")]
    pub sell_rate: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Balance {
    pub currency: String,
    #[serde(rename = "balanceTotal")]
    pub balance_total: String,
    #[serde(rename = "balanceAvailable")]
    pub balance_available: String,
    #[serde(rename = "balanceReserved")]
    pub balance_reserved: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Orderbook {
    #[serde(rename = "currencyPair")]
    pub currency_pair: String,
    pub bids: Vec<OrderbookEntry>,
    pub asks: Vec<OrderbookEntry>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct OrderbookEntry {
    #[serde(deserialize_with="str_to_f64")]
    pub price: f64,
    #[serde(rename = "volume")]
    pub volume: String,
    #[serde(rename = "valueInOppositeCurrency")]
    pub value_in_opposite_currency: String,
}

fn str_to_f64<'de, D>(deserializer: D,) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
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

pub struct DirectOrderRequest<'a> {
    dry_run: &'a str,
    submit_id: &'a str,
    currency_pair: &'a str,
    buy_sell: &'a str,
    volume: &'a str,
    volume_currency: &'a str,
    ts: &'a str,
}

pub struct MarketOrderRequest<'a> {
    dry_run: &'a str,
    submit_id: &'a str,
    currency_pair: &'a str,
    buy_sell: &'a str,
    volume: &'a str,
    volume_currency: &'a str,
    limit_price: &'a str,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct MakeOrderResponse {
    duplicate: Option<bool>,
    #[serde(rename = "orderId")]
    order_d: Option<String>,
    message: Option<String>,
    errors: Option<HashMap<String, Vec<String>>>,
}

pub struct PayoutRequest {

}

pub struct PayoutResponse {

}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Order {} {} {} {}", self.order_id, self.status, self.market, self.buy_sell)
    }
}

#[cfg(test)]
mod tests {
}
