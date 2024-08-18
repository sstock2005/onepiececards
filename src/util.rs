
// util functions that make life easier

// https://doc.rust-lang.org/book/ch12-02-reading-a-file.html
// https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html

use std::{fs, collections::HashMap, time::UNIX_EPOCH};
use base64::{prelude::BASE64_STANDARD, Engine};
use serde_json::{Number, Value};
use std::time::SystemTime;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Database
{
    pub cards: Vec<Card>
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Card 
{
    pub product_id: Number
}

pub fn read_file(path: &str) -> String
{
    let result = fs::read_to_string(path);
    let content = match result {
        Ok(file) => file,
        Err(error) => error.to_string(),
    };

    return content;
}

pub fn write_file(path: &str, data: &str) -> bool
{
    let op = fs::write(path, data);
    let result = match op {
        Ok(_) => true,
        Err(_) => false
    };

    return result;
}

pub fn import(path: &str) -> Database {
    let data = read_file(path);
    let db_result = serde_json::from_str(data.as_str());

    let db = match db_result {
        Ok(db) => db,
        Err(error) => {
            println!("[!!] Could not import database!\n[!!] {}", error.to_string());
            println!("[!!] Using empty database... This will overwrite your current one if you have it saved!");
            pause_console::pause_console!();
            Database { cards: Vec::new() }
        }
    };

    return db;
}

pub fn save_db(db: Database, db_path: &str) -> bool
{
    let json = serde_json::to_string(&db).unwrap();
    return write_file(db_path, &json);
}

pub fn clear_cache(cache_path: &str) -> HashMap<String, String>
{
    let new_webcache: HashMap<String, String> = HashMap::new();

    let _ = save_cache(new_webcache.clone(), &cache_path);

    return new_webcache;
}

pub fn save_cache(webcache: HashMap<String, String>, cache_path: &str) -> bool 
{
    let json_request = serde_json::to_string(&webcache);

    match json_request
    {
        Ok(str) => write_file(cache_path,&str),
        Err(_) => false
    }
}

pub fn import_cache(cache_path: &str) -> HashMap<String, String> {
    let content = read_file(cache_path);

    let json_request: Result<HashMap<String, String>, serde_json::Error> = serde_json::from_str(&content);

    match json_request
    {
        Ok(webcache) => return webcache,
        Err(_) => return HashMap::new()
    }
}

pub fn cache_old(path: &str) -> u64 {

    let mut file_since_epoch: u64 = u64::MAX;

    match fs::metadata(path)
    {
        Ok(file) => file_since_epoch = file.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        Err(_) => ()
    };

    let start = SystemTime::now();
    
    let current_since_epoch = start
    .duration_since(UNIX_EPOCH).unwrap().as_secs();

    if file_since_epoch == u64::MAX 
    {
        return file_since_epoch;
    }

    return (current_since_epoch / 60 / 60) - (file_since_epoch / 60 / 60)
}

pub fn cache_too_old(cache_old: u64) -> bool
{
    cache_old > 24
}

pub fn clear(webcache: HashMap<String, String>)
{
    print!("{esc}c", esc = 27 as char);
    println!("###################");
    println!("#     card db     #");
    println!("###################\n");
    println!("Cache: {} requests\n", webcache.len());
}

pub fn format_id(id: Number) -> String
{
    format!("{}", id).replace(".0", "")
}

pub fn pause()
{
    pause_console::pause_console!("");
    pause_console::pause_console!();
}

pub fn check_cache(method_params: String, webcache: &HashMap<String, String>) -> Option<String>
{
    let result = webcache.get(&method_params);
    return result.cloned();
}

pub fn get_product_details(formatted_product_id: &str, mut webcache: HashMap<String, String>) -> (String, HashMap<String, String>)
{
    let cache_result = check_cache(format!("get_product_details:{}", formatted_product_id), &webcache);

    if cache_result != None
    {
        return (cache_result.unwrap(), webcache);
    }

    let client = reqwest::blocking::Client::builder().build().unwrap();

    let request = client.request(reqwest::Method::GET, format!("https://mp-search-api.tcgplayer.com/v2/product/{}/details", formatted_product_id));

    let response = request.send().unwrap();
    let body = response.text().unwrap();

    webcache.insert(format!("get_product_details:{}", formatted_product_id), body.clone());

    return (format!("{}", body), webcache);
}

pub fn search(card_name: &str, mut webcache: HashMap<String, String>) -> (String, HashMap<String, String>)
{
    let cache_result = check_cache(format!("search:{}", card_name), &webcache);

    if cache_result != None
    {
        return (cache_result.unwrap(), webcache);
    }

    let client = reqwest::blocking::Client::builder()
        .build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let data = r#"{"algorithm":"sales_synonym_v2","from":0,"size":24,"filters":{"term":{},"range":{},"match":{}},"listingSearch":{"context":{"cart":{}},"filters":{"term":{"sellerStatus":"Live","channelId":0},"range":{"quantity":{"gte":1}},"exclude":{"channelExclusion":0}}},"context":{"cart":{},"shippingCountry":"US","userProfile":{}},"settings":{"useFuzzySearch":true,"didYouMean":{}},"sort":{}}"#;

    let json: serde_json::Value = serde_json::from_str(&data).unwrap();

    let request = client.request(reqwest::Method::POST, 
        format!("https://mp-search-api.tcgplayer.com/v1/search/request?q={}", card_name))
        .headers(headers)
        .json(&json);

    let response = request.send().unwrap();
    let body = response.text().unwrap();

    webcache.insert(format!("search:{}", card_name), body.clone());

    return (format!("{}", body), webcache);
}

pub fn card_image_b64(formatted_product_id: &str, mut webcache: HashMap<String, String>) -> (String, HashMap<String, String>)
{
    let cache_result = check_cache(format!("card_image_b64:{}", formatted_product_id), &webcache);

    if cache_result != None
    {
        return (cache_result.unwrap(), webcache);
    }

    let client = reqwest::blocking::Client::builder().build().unwrap();

    let request = client.request(reqwest::Method::GET, format!("https://tcgplayer-cdn.tcgplayer.com/product/{}_in_1000x1000.jpg", formatted_product_id));

    let response = request.send().unwrap();
    let bytes = response.bytes().unwrap();

    webcache.insert(format!("card_image_b64:{}", formatted_product_id), BASE64_STANDARD.encode(bytes.clone()));
    
    return (BASE64_STANDARD.encode(bytes), webcache);
}

pub fn total_worth(db: &Database, webcache: HashMap<String, String>) -> (f64, HashMap<String, String>) {
    db.cards.iter().fold((0.0, webcache.clone()), |acc: (f64, HashMap<String, String>), product| {
        let product_info: Value = serde_json::from_str(
            &get_product_details(&format_id(product.product_id.clone()), webcache.clone()).0
        ).unwrap();
        
        (acc.0 + product_info["marketPrice"].as_f64().unwrap_or(0.0), acc.1)
    })
}