use std::{collections::HashMap, thread, time::Duration};

use reqwest::blocking::Client;
use serde_json::{json, Value};

fn main() {
    let j = thread::spawn(|| {
        vesteda();
    });

    let k = thread::spawn(|| {
        rebo();
    });

    let v = vec![j, k];

    for handle in v {
        handle.join();
    }
}

fn vesteda() {
    let mut house_keeper = HouseKeeper {
        n_houses: 0,
        list: vec![],
    };

    let client = reqwest::blocking::Client::new();

    // let mut placeObject = HashMap::new();
    // placeObject.insert("name", "Zutphen, Nederland");
    // placeObject.insert("placeType", "1");
    // placeObject.insert("lng", "6.19605827");
    // placeObject.insert("lat", "52.1427345");

    // let mut json = HashMap::new();
    // json.insert("place", "Zutphen, Nederland");
    // json.insert("placeObject", placeObject);
    // headers.insert("location", "Zutphen");

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
	"radius": 10});
    // let str: String = String::from(json.as_str().unwrap());
    // let sstr = str.unwrap();

    loop {
        println!("pinging vesteda");
        let res = client
            .post("https://www.vesteda.com/api/units/search")
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
            if house_keeper.list.iter().any(|hk| hk["id"] == item["id"]) {
                println!("house already known");
            } else {
                println!("house not known! notify!");
                send_telegram(
                    (
                        item["street"].as_str().unwrap(),
                        item["houseNumber"].as_str().unwrap_or("0"),
                        item["url"].as_str().unwrap(),
                    ),
                    &client,
                );
            }
        }

        house_keeper.list = found;

        thread::sleep(Duration::from_secs(150));
    }
}

fn rebo() {
    let mut house_keeper = HouseKeeper {
        n_houses: 0,
        list: vec![],
    };

    let client = reqwest::blocking::Client::new();

    let mut headers = HashMap::new();
    headers.insert("location", "Zutphen");

    loop {
        println!("pinging rebo");

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

        for item in &found {
            if house_keeper.list.iter().any(|hk| hk["id"] == item["id"]) {
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
                    &client,
                );
            }
        }

        house_keeper.list = found;

        thread::sleep(Duration::from_secs(150));
    }
}

pub struct HouseKeeper {
    pub n_houses: usize,
    pub list: Vec<Value>,
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
