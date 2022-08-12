use std::{collections::HashMap, thread, time::Duration};

use reqwest::blocking::Client;
use serde_json::Value;

fn main() {
    let mut house_keeper = HouseKeeper {
        n_houses: 0,
        list: vec![],
    };

    let client = reqwest::blocking::Client::new();

    let mut headers = HashMap::new();
    headers.insert("location", "Zutphen");

    loop {
        let res = client
            .post("https://www.rebohuurwoning.nl/object/search/list")
            .form(&headers)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let v: Value = serde_json::from_str(&res).unwrap();
        let v_items = v["data"]["items"].clone();

        let v_arr = v_items.to_owned();
        let v_arr = v_arr.as_array().cloned().unwrap();

        let found: Vec<_> = v_arr
            .into_iter()
            .filter(|h| h["city"] == "Zutphen")
            .collect();
        if found.ne(&house_keeper.list) {
            house_keeper.list = found;
            house_keeper.n_houses = house_keeper.list.len();
            println!("the list changed, send it");
            send_telegram(&house_keeper, &client)
        }

        thread::sleep(Duration::from_secs(200));
    }
}

pub struct HouseKeeper {
    pub n_houses: usize,
    pub list: Vec<Value>,
}

pub fn send_telegram(house_keeper: &HouseKeeper, client: &Client) {
    let s_url = format!(
        "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={} woningen",
        std::env::var("TELEGRAM_BOT_TOKEN").unwrap(),
        std::env::var("TELEGRAM_USER_ID").unwrap(),
        house_keeper.n_houses
    );
    let _response = client.get(&s_url).send().unwrap();

    for v in &house_keeper.list {
        let s_url = format!(
            "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text=NIEUWE WONING:%0A{} {}%0A%0Ahttps://rebohuurwoning.nl{}",
            std::env::var("TELEGRAM_BOT_TOKEN").unwrap(),
            std::env::var("TELEGRAM_USER_ID").unwrap(),
            v["street"].as_str().unwrap(),
            v["street_number"].as_str().unwrap(),
            v["object_url"].as_str().unwrap()
        );
        let _response = client.get(&s_url).send().unwrap();
    }
}
