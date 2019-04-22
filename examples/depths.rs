extern crate walutomat;

use std::{thread, time};

fn main() {
  let wt = walutomat::v1::API::new("https://api.walutomat.pl", "", "");

  let one_sec = time::Duration::from_secs(1);

  let currencies = ["EUR", "GBP", "USD", "CHF", "PLN"];
  let mut pairs: Vec<String> = vec![];
  for i in 0..currencies.len()-1 {
    for j in i+1..currencies.len() {
      pairs.push(format!("{}_{}", currencies[i], currencies[j]));
    }
  }

  let mut i = 0;
  loop {
    if i == 0 {
      println!("{}", pairs.join(" "));
    }
    i = (i + 1) % 20;
    let spreads = pairs.iter().map(|p| {
      let orderbook = wt.get_orderbook(&p).unwrap();
      format!("{}/{}", &orderbook.asks[0].base_volume, &orderbook.bids[0].base_volume)
    });
    println!(" {}", spreads.collect::<Vec<String>>().join("  "));
    thread::sleep(one_sec);
  }
}
