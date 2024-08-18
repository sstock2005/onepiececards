use std::{collections::HashMap, process::Command, thread, time};

use pause_console::pause_console;
use serde_json::Value;
use text_io::read;
use util::{format_id, pause, save_db, Card, Database};

mod util;
mod html;

const DB_LOC: &str = "card.json";
const CACHE_LOC: &str = "webcache.dat";

fn main()
{
    println!("[-] loading webcache");
    thread::sleep(time::Duration::from_millis(300));
    
    let mut webcache: HashMap<String, String> = util::import_cache(CACHE_LOC);

    let cache_time = util::cache_old(CACHE_LOC);

    if cache_time == u64::MAX
    {
        webcache = HashMap::new();
    }
    
    else if util::cache_too_old(cache_time)
    {
        println!("[!] cache is more than a day old! Would you like to wipe it? (y/n)");
        let confirmation: char = read!();

        if confirmation == 'y'
        {
            webcache = HashMap::new();
            println!("[-] cache wiped!");
            thread::sleep(time::Duration::from_millis(300));
        }
    }

    // invalid cache
    if webcache.is_empty()
    {
        webcache = HashMap::new();
    }

    println!("[+] loaded webcache");
    thread::sleep(time::Duration::from_millis(300));

    println!("[-] loading card data");
    thread::sleep(time::Duration::from_millis(300));

    let db = util::import(DB_LOC);
    println!("[+] loaded card data");
    thread::sleep(time::Duration::from_millis(300));

    menu(db, webcache);
}

fn menu(mut db: Database, mut webcache: HashMap<String, String>) -> ()
{
    loop
    {
        util::clear(webcache.clone());

        println!("Loading...");

        let mut output = String::new();
        output.push_str("| Pos | Name | ID | Market Price |\n");

        let mut i: u32 = 0;
        for product in db.cards.clone()
        {
            util::clear(webcache.clone());
        
            let percent = format!("{:.2}%", (i as f64 / db.cards.clone().len() as f64) * 100.0);
            println!("Loading... {}", percent);

            let product_util_request = util::get_product_details(&format_id(product.product_id.clone()), webcache.clone());
            webcache = product_util_request.1;
            
            let product_info: Value = serde_json::from_str(product_util_request.0.as_str()).unwrap();
            let mut product_name = product_info["productName"].as_str().unwrap().to_string();
            let set_url_name = product_info["setUrlName"].as_str().unwrap();

            let option_op_code = product_info["customAttributes"]["number"].as_str();

            let op_code = match option_op_code{
                None => "",
                Some(code) => code
            };

            if set_url_name.contains("Pre Release")
            {
                product_name = format!("{} {} (Pre Release)", product_name, op_code);
            }
            else
            {
                product_name = format!("{} {}", product_name, op_code);
            }

            let market_price_result = product_info["marketPrice"].as_number();
            let market_price: String = match market_price_result{
                None => "error".to_string(),
                Some(price) =>{
                    let string = format!("${}", price).as_str().to_owned();
                    string
                }
            };

            output.push_str(format!("| {} | {} | {} | {} |\n", i, product_name, format_id(product.product_id), market_price).as_str());
            i += 1;
        }

        util::clear(webcache.clone());

        println!("{}\n", output);
        println!("[1] add new card [2] remove a card [3] generate card report [4] clear cache [5] quit");
        let input: i32 = read!();
        match input
        {
            1 => (db, webcache) = add_card(db.clone(), webcache.clone()),
            2 => (db, webcache) = remove_card(db.clone(), webcache.clone()),
            3 => (db, webcache) = generate_report(db.clone(), webcache.clone()),
            4 => webcache = util::clear_cache(CACHE_LOC),
            5 => quit(db.clone(), webcache.clone()),
            _ => { pause_console!("Incorrect Option! Hit Enter to try again!"); }
        };
    }
}

fn add_card(mut db: Database, mut webcache: HashMap<String, String>) -> (Database, HashMap<String, String>)
{
    util::clear(webcache.clone());

    println!("Input Card Name:");
    let input: String = read!("\n{}\n");
    let util_result = util::search(&input, webcache.clone());
    webcache = util_result.1;
    let result = util_result.0;

    let request: Value = serde_json::from_str(&result).unwrap();
    let products = request["results"][0]["results"].as_array().unwrap();
    
    util::clear(webcache.clone());
    println!("Select Correct Card (ID:COUNT) (eg. 0:1 for 1 of 0):");

    let mut i: u32 = 0;
    let mut hit: bool = false;

    for product in products
    {
        let product_line_name = product["productLineName"].as_str().unwrap();
        let option_op_code = product["customAttributes"]["number"].as_str();
        let set_url_name = product["setUrlName"].as_str().unwrap();

        let op_code = match option_op_code{
            None => "",
            Some(code) => code
        };

        if op_code == "" || product_line_name != "One Piece Card Game"
        {
            i += 1;
            continue;
        }

        let name = &product["productName"];
        let mut name_string = name.as_str().unwrap().to_string();

        if set_url_name.contains("Pre Release")
        {
            name_string = format!("{} {} (Pre Release)", name_string, op_code);
        }
        else
        {
            name_string = format!("{} {}", name_string, op_code);
        }

        println!("[{}] {}", i, name_string);
        hit = true;
        i += 1;
    }
    
    if !hit
    {
        println!("No results! Try searching the name in a different way!");
        pause_console::pause_console!();
        return (db, webcache);
    }

    print!("\nSelection (ID:COUNT) (eg. 0:1 for 1 of 0): ");
    
    let selection_string: String = read!("{}\n");

    let selection_vec = selection_string.trim().split(':').collect::<Vec<&str>>();

    let selection_attempt = selection_vec[0].parse::<usize>();

    let selection = match selection_attempt
    {
        Ok(usize) => usize,
        Err(_) => { 
            println!("Incorrect Selection Format!");  

            let error: usize = usize::MAX;

            error
        }
    };
    
    if selection == usize::MAX
    {
        pause_console::pause_console!();
        return (db, webcache);
    }

    let count_attempt = selection_vec[1].parse::<u32>();

    let count = match count_attempt
    {
        Ok(u32) => u32,
        Err(_) => { 
            println!("Incorrect Count Format!");  

            let error: u32 = u32::MAX;

            error
        }
    };
    
    if count == u32::MAX
    {
        pause_console::pause_console!();
        return (db, webcache);
    }

    let mut j: u32 = 0;

    while j < count
    {
        db.cards.push(Card { product_id: products[selection]["productId"].as_number().unwrap().clone() });
        j += 1;
    }

    let save_result = save_db(db.clone(), DB_LOC);

    if !save_result
    {
        println!("[debug] [error] Database could not be saved!");
        pause();
    }

    (db, webcache)
}

fn remove_card(mut db: Database, mut webcache: HashMap<String, String>) -> (Database, HashMap<String, String>)
{
    print!("Selection: ");
    
    let selection: usize = read!();

    let product_info_util_request = util::get_product_details(&format_id(db.cards[selection].product_id.clone()), webcache.clone());
    webcache = product_info_util_request.1;

    let product_info: Value = serde_json::from_str(product_info_util_request.0.as_str()).unwrap();
    let mut product_name = product_info["productName"].as_str().unwrap().to_string();
    let set_url_name = product_info["setUrlName"].as_str().unwrap();

    let option_op_code = product_info["customAttributes"]["number"].as_str();

    let op_code = match option_op_code{
        None => "",
        Some(code) => code
    };

    if set_url_name.contains("Pre Release")
    {
        product_name = format!("{} {} (Pre Release)", product_name, op_code);
    }
    else
    {
        product_name = format!("{} {}", product_name, op_code);
    }

    println!("Are you sure you want to delete {}? (y/n)", product_name);

    let confirmation: char = read!();

    if confirmation == 'y'
    {
        db.cards.remove(selection);

        let save_result = save_db(db.clone(), DB_LOC);

        if !save_result
        {
            println!("[debug] [error] Database could not be saved!");
            pause();
        }
    }

    (db, webcache)
}

fn quit(_db: Database, webcache: HashMap<String, String>)
{
    util::save_cache(webcache, CACHE_LOC);
    std::process::exit(0);
}

fn generate_report(db: Database, mut webcache: HashMap<String, String>) -> (Database, HashMap<String, String>)
{
    let generate_result = html::generate(db.clone(), webcache.clone());

    webcache = generate_result.1;

    let result = generate_result.0;

    util::clear(webcache.clone());

    if result
    {
        util::save_cache(webcache.clone(), CACHE_LOC);
        println!("generated a report! open report.html? (y/n)");

        let confirmation: char = read!();

        if confirmation == 'y'
        {
            let _ = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/C", "start report.html"])
                    .spawn()
                    .unwrap()
            } else {
                Command::new("xdg-open")
                    .arg("report.html")
                    .spawn()
                    .unwrap()
            };
        }
    }
    else
    {
        println!("[debug] [error] could not generate a report!");
    }

    pause();

    (db, webcache)
}