walutomat-rs is a Rust library for interacting with Walutomat API.

```rust
extern crate walutomat;

fn main() {
  let wt = walutomat::V1::new("https://api.walutomat.pl", "key", "secret");
  let orderbook = wt.get_orderbook("EUR_PLN").unwrap();
  
  println!("{}", orderbook.pair);
  for entry in orderbook.asks.iter().zip(orderbook.bids) {
    println!("{} {}", entry.0.price, entry.1.price);
  }
}


```