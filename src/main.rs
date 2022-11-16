use std::{collections::HashMap, thread, time::Duration};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use reqwest::{blocking::Client, header::*, Url};
use serde_json::{json, Value};

fn main() {
    logging_setup();

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

    let mut nmg = Makelaar::NMG(Agent {
        base_url: Url::parse("https://nmgwonen.nl/huur/").unwrap(),
        client: reqwest::blocking::Client::new(),
        houses: vec![],
    });

    let mut handles = vec![];

    handles.push(thread::spawn(move || loop {
        vesteda.query();
        thread::sleep(Duration::from_secs(775));
    }));

    handles.push(thread::spawn(move || loop {
        rebo.query();
        thread::sleep(Duration::from_secs(776));
    }));

    handles.push(thread::spawn(move || loop {
        nmg.query();
        thread::sleep(Duration::from_secs(777));
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
    NMG(Agent),
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
                        debug!("house already known");
                    } else {
                        info!("house not known! notify!");
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
                        debug!("house already known");
                    } else {
                        info!("house not known! notify!");
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
            Makelaar::NMG(agent) => {
                let mut form_fields = HashMap::new();
                form_fields.insert("__live", "1");
                form_fields.insert("adres_plaats_postcode", "zutphen");
                form_fields.insert("__maps", "paged");

                let mut header_map = HeaderMap::new();
                header_map.insert(
                    CONTENT_TYPE,
                    HeaderValue::from_str(
                        "multipart/form-data; boundary=---011000010111000001101001",
                    )
                    .unwrap(),
                );

                // agent.houses = vec![json!({"total": 0})];

                let res = agent
                    .client
                    .post(agent.base_url.as_ref())
                    .headers(header_map)
                    .form(&form_fields)
                    .send()
                    .unwrap()
                    .text()
                    .unwrap();

                let v: Value = serde_json::from_str(&res).unwrap();

                if let Some(total) = v["total"].as_i64() {
                    if let Some(old_total) = agent.houses.first() {
                        if old_total != total {
                            info!("house not known! notify!");
                            send_telegram(("Onbekend ivm nmg", "na", "https://nmgwonen.nl/huur/#q1ZKTClKLY4vyElMLAFS-cUlyfkpqUpWSlWlJQUZqXlKOkoFiempxUCRjNLSIqVaAA"), &agent.client);

                            let json = json!({ "total": total });
                            agent.houses = vec![json]
                        } else {
                            debug!("house already known");
                        }
                    } else {
                        let json = json!({ "total": total });
                        agent.houses = vec![json]
                    }
                }
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

fn logging_setup() {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    let _ = color_eyre::install();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut nmg = Makelaar::NMG(Agent {
            base_url: Url::parse("https://nmgwonen.nl/huur/").unwrap(),
            client: reqwest::blocking::Client::new(),
            houses: vec![],
        });

        nmg.query();
    }
}
