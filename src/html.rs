use std::collections::HashMap;

use serde_json::{Number, Value};

use crate::util::{self, format_id, Database};

pub fn generate(db: Database, mut webcache: HashMap<String, String>) -> (bool, HashMap<String, String>)
{
    let total_worth = util::total_worth(&db, webcache.clone());

    webcache = total_worth.1;

    let html_code_start = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>One Piece TCG Card List</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            background-color: #0a192f;
            color: #e6f1ff;
            margin: 0;
            padding: 0;
        }}

        .header {{
            background-color: #112240;
            padding: 10px 20px;
            text-align: center;
            position: sticky;
            top: 0;
            z-index: 1000;
        }}

        .header h2 {{
            color: #64ffda;
            margin: 0;
        }}

        h1 {{
            text-align: center;
            color: #64ffda;
            padding: 20px 0;
        }}

        .card-container {{
            display: flex;
            flex-wrap: wrap;
            justify-content: center;
            gap: 20px;
            padding: 20px;
        }}

        .card {{
            background-color: #112240;
            border-radius: 10px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            display: flex;
            width: 100%;
            max-width: 600px;
            overflow: hidden;
            flex-direction: row;
        }}

        .card-image {{
            width: 200px;
            height: 280px;
            object-fit: cover;
        }}

        .card-info {{
            padding: 20px;
            flex-grow: 1;
            display: flex;
            flex-direction: column;
        }}

        .card-name {{
            font-size: 24px;
            font-weight: bold;
            margin-bottom: 10px;
            color: #64ffda;
            word-wrap: break-word; /* Allow long words to break */
            overflow-wrap: break-word; /* Alternative property for better browser support */
        }}

        .card-description {{
            margin-bottom: 15px;
        }}

        .card-value {{
            font-weight: bold;
            color: #5ccfee;
        }}

        @media (max-width: 600px) {{
            .card {{
                flex-direction: column;
            }}

            .card-image {{
                width: 100%;
                height: auto;
                max-height: 280px;
            }}

            .card-name {{
                font-size: 20px; /* Slightly reduce font size on small screens */
            }}
        }}

        .search-container {{
            text-align: center;
            margin: 20px 0;
        }}

        #searchInput {{
            padding: 10px;
            width: 300px;
            border-radius: 5px;
            border: none;
            font-size: 16px;
        }}

        #sortSelect {{
            padding: 10px;
            margin-left: 10px;
            border-radius: 5px;
            border: none;
            font-size: 16px;
            background-color: #112240;
            color: #e6f1ff;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h2>Total Market Worth: ${:.2}</h2>
    </div>
    <h1>One Piece TCG Card List</h1>
    <div class="search-container">
        <input type="text" id="searchInput" placeholder="Search card names...">
        <select id="sortSelect">
            <option value="default">Sort: Default</option>
            <option value="market-value">Sort: Market Value</option>
            <option value="a-z">Sort: A-Z</option>
        </select>
    </div>
    <div class="card-container" id="cardContainer">
"#, total_worth.0);

let html_code_end = r#"</div>
<script>
    const searchInput = document.getElementById('searchInput');
    const sortSelect = document.getElementById('sortSelect');
    const cardContainer = document.getElementById('cardContainer');
    const cards = Array.from(cardContainer.children);

    function sortCards() {
        const searchTerm = searchInput.value.toLowerCase();
        const sortOption = sortSelect.value;

        cards.sort((a, b) => {
            const aName = a.querySelector('.card-name').textContent.toLowerCase();
            const bName = b.querySelector('.card-name').textContent.toLowerCase();
            const aMatch = aName.includes(searchTerm);
            const bMatch = bName.includes(searchTerm);
            
            if (aMatch && !bMatch) return -1;
            if (!aMatch && bMatch) return 1;

            switch (sortOption) {
                case 'market-value':
                    const aValue = parseFloat(a.querySelector('.card-value').textContent.replace('Market Value: $', ''));
                    const bValue = parseFloat(b.querySelector('.card-value').textContent.replace('Market Value: $', ''));
                    return bValue - aValue;
                case 'a-z':
                    return aName.localeCompare(bName);
                default:
                    return 0;
            }
        });
        
        cardContainer.innerHTML = '';
        cards.forEach(card => cardContainer.appendChild(card));
    }

    searchInput.addEventListener('input', sortCards);
    sortSelect.addEventListener('change', sortCards);

    sortCards();
</script>
</body>
</html>"#;

let mut html_code = String::new();

html_code.push_str(&html_code_start);

let mut i: u32 = 0;
for product in &db.cards
{
    util::clear(webcache.clone());

    let percent = format!("{:.2}%", (i as f64 / db.cards.clone().len() as f64) * 100.0);
    println!("generating report... {}", percent);

    let product_info_request = util::get_product_details(&format_id(product.product_id.clone()), webcache.clone());

    webcache = product_info_request.1;

    let product_info: Value = serde_json::from_str(product_info_request.0.as_str()).unwrap();
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

    let product_description_request = product_info["customAttributes"]["description"].as_str();
    let product_description: String = match product_description_request{
        None => "No description provided.".to_string(),
        Some(description) => description.replace("<", " <").replace(">", "> ")
    };

    let market_value_request = product_info["marketPrice"].as_number();

    let market_value = match market_value_request{
        None => Number::from_f64(0.0).unwrap(),
        Some(num) => num.to_owned()
    };

    let image_b64_request = util::card_image_b64(&util::format_id(product.product_id.clone()), webcache.clone());

    webcache = image_b64_request.1;
    
    let image_b64 = image_b64_request.0;
    let product_url = format!("https://www.tcgplayer.com/product/{}", format_id(product.product_id.clone()));

    html_code.push_str(&generate_card(&product_url, &product_name, market_value.clone(), &image_b64, &product_description));
    i += 1;
}

html_code.push_str(&html_code_end);
return (util::write_file("report.html", &html_code), webcache.clone());
}

fn generate_card(product_url: &str, product_name: &str, market_value: Number, image_b64: &str, product_description: &str) -> String
{
    let html_card = format!(r#"<div class="card">
            <img src="data:image/jpeg;base64,{}" alt="card" class="card-image">
            <div class="card-info">
                <a href={} target="_blank"><div class="card-name">{}</div></a>
                <div class="card-description">{}</div>
                <div class="card-value">Market Value: ${}</div>
            </div>
        </div>"#, image_b64, product_url, product_name, product_description, market_value);
    
    return html_card;
}