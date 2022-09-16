use std::{thread, time::Duration};

use reqwest::{blocking::Client, Url};
use serde_json::{json, Value};

fn main() {
    let mut vesteda = Makelaar::Vesteda(Agent {
        base_url: Url::parse("https://www.vesteda.com/api/units/search").unwrap(),
        client: reqwest::blocking::Client::new(),
        houses: vec![],
    });

    let mut rebo = Makelaar::Rebo(Agent {
        base_url: Url::parse("https://www.rebohuurwoning.nl/object/search/list").unwrap(),
        client: reqwest::blocking::Client::new(),
        houses: vec![],
    });

    let mut handles = vec![];

    handles.push(thread::spawn(move || {
        vesteda.query();
        thread::sleep(Duration::from_secs(1800));
    }));

    handles.push(thread::spawn(move || {
        rebo.query();
        thread::sleep(Duration::from_secs(1800));
    }));

    for handle in handles {
        let _ = handle.join();
    }
}

pub struct Agent {
    base_url: Url,
    client: reqwest::blocking::Client,
    houses: Vec<Value>,
}

pub enum Makelaar {
    Vesteda(Agent),
    Rebo(Agent),
}

trait Queryable {
    fn query(&mut self);
}

impl Queryable for Makelaar {
    fn query(&mut self) {
        match self {
            Makelaar::Rebo(agent) => {
                let res = agent
                    .client
                    .post(agent.base_url.as_ref())
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

                for item in &found {
                    if agent.houses.iter().any(|hk| hk["id"] == item["id"]) {
                        println!("house already known");
                    } else {
                        println!("house not known! notify!");
                        send_telegram(
                            (
                                item["street"].as_str().unwrap(),
                                item["street_number"].as_str().unwrap(),
                                &format!(
                                    "{}{}",
                                    "https://rebohuurwoning.nl",
                                    item["object_url"].as_str().unwrap()
                                ),
                            ),
                            &agent.client,
                        );
                    }
                }

                agent.houses = found;
            }
            Makelaar::Vesteda(agent) => {
                let json = json!({
                    "place": "Zutphen, Nederland",
                    "placeObject": {
                        "name": "Zutphen, Nederland",
                        "placeType": "1",
                        "lng": "6.19605827",
                        "lat": "52.1427345"
                    },
                    "placeType": 1,
                    "priceFrom": 500,
                    "priceTo": 9999,
                    "sizes": [],
                    "sortType": 0,
                    "bedRooms": 0,
                    "rootId": 1303,
                    "unitTypes": [
                        2,
                        1,
                        4
                    ],
                    "other": [],
                    "lat": "52.1427345",
                    "lng": "6.19605827",
                    "radius": 10
                });

                let res = agent
                    .client
                    .post(agent.base_url.as_ref())
                    .json(&json)
                    .send()
                    .unwrap()
                    .text()
                    .unwrap();

                let v: Value = serde_json::from_str(&res).unwrap();
                let v_items = v["items"].clone();

                let v_arr = v_items.to_owned();
                let found = v_arr.as_array().cloned().unwrap();

                for item in &found {
                    if agent.houses.iter().any(|hk| hk["id"] == item["id"]) {
                        println!("house already known");
                    } else {
                        println!("house not known! notify!");
                        send_telegram(
                            (
                                item["street"].as_str().unwrap(),
                                item["houseNumber"].as_str().unwrap_or("0"),
                                item["url"].as_str().unwrap(),
                            ),
                            &agent.client,
                        );
                    }
                }

                agent.houses = found;
            }
        }
    }
}

pub fn send_telegram(house: (&str, &str, &str), client: &Client) {
    let s_url =
        format!(
        "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text=NIEUWE WONING: {} {}%0A%0A{}",
        std::env::var("TELEGRAM_BOT_TOKEN").unwrap(),
        std::env::var("TELEGRAM_USER_ID").unwrap(),
        house.0, house.1, house.2
    );
    let _response = client.get(&s_url).send().unwrap();
}
