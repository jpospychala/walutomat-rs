walutomat-rs is a Rust library for interacting with Walutomat API.

* Currency exchange on P2P currency market
* Currency exchange with CurrencyOne, at guaranteed price
* International transfers

```rust
extern crate walutomat;

fn main() {
  let wt = walutomat::v2::API::new("https://api.walutomat.pl", "key");
  let orderbook = wt.market_fx_best_offers("EURPLN").unwrap().result.unwrap();
  
  println!("{}", orderbook.pair);
  for entry in orderbook.asks.iter().zip(orderbook.bids) {
    println!("{} {}", entry.0.price, entry.1.price);
  }
}


```