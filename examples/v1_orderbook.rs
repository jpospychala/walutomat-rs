extern crate walutomat;

fn main() {
  let client = walutomat::v1::Client::new("https://api.walutomat.pl", "", "");
  let orderbook = client.get_orderbook("EUR_PLN").unwrap();
  
  println!("{}", orderbook.pair);
  for entry in orderbook.asks.iter().zip(orderbook.bids) {
    println!("{} {}", entry.0.price, entry.1.price);
  }
}
