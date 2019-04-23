extern crate walutomat;

use std::env;

fn main() {
  let client = walutomat::v2::Client::new("https://api.walutomat.pl", &env::var("WT_KEY").unwrap());
  let balance = client.account_balance();
  for b in balance.unwrap().result.unwrap() {
    println!("{} {} (blocked {})", b.currency, b.balance_total, b.balance_reserved);
  }
}
