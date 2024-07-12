use std::{env, fs, io::Write};

use rand::random;
use reqwest::{Client, RequestBuilder};
use scraper::{selectable::Selectable, ElementRef, Html, Selector};
use serde::Serialize;
use tokio::task::{self, JoinHandle};

#[derive(Serialize, Debug, Clone)]
struct SportVisionProduct {
    image_url: String,
    image_url_high_res: String,
    brand_name: String,
    title: String,
    short_description: String,
    current_price: String,
    id: String,
    stock: u8,
}

impl SportVisionProduct {
    fn from_div(div: ElementRef) -> SportVisionProduct {
        let mut image_url = String::from(
            div.select(&Selector::parse(".img-wrapper img").unwrap())
                .next()
                .unwrap()
                .attr("src")
                .unwrap()
                .trim(),
        );
        image_url.insert_str(0, "https://sportvision.rs");
        let image_url_high_res = image_url
            .replace("thumbs_350", "thumbs_800")
            .replace("350_350px", "800_800px");
        let id = String::from(
            div.select(&Selector::parse(".text-wrapper .category-wrapper span").unwrap())
                .next()
                .unwrap()
                .text()
                .next()
                .unwrap()
                .trim(),
        );
        let stock: u8 = random::<u8>() / 2;
        let brand_name = (if let Some(innerr) = div
            .select(&Selector::parse(".text-wrapper .brand a").unwrap())
            .next()
        {
            innerr.inner_html()
        } else {
            "".to_string()
        })
        .trim()
        .to_string();
        let title = String::from(
            div.select(&Selector::parse(".text-wrapper .title a").unwrap())
                .next()
                .unwrap()
                .text()
                .nth(1)
                .unwrap()
                .trim(),
        );
        let short_description = String::from(
            div.select(&Selector::parse(".product-shortname").unwrap())
                .next()
                .unwrap()
                .text()
                .next()
                .unwrap_or_default()
                .trim(),
        );
        let current_price = String::from(
            div.select(&Selector::parse(".prices-wrapper .current-price").unwrap())
                .next()
                .unwrap()
                .text()
                .nth(1)
                .unwrap()
                .trim(),
        );
        SportVisionProduct {
            image_url,
            image_url_high_res,
            brand_name,
            title,
            short_description,
            current_price,
            id,
            stock,
        }
    }
}

#[tokio::main]
async fn main() {
    let last_page: u16 = env::args()
        .nth(1)
        .expect("Expected 1 argument, got 0")
        .parse()
        .expect("Argument should be a non-negative number");

    let mut products: Vec<SportVisionProduct> = vec![];
    let klijent = Client::new();
    let mut join_handles: Vec<JoinHandle<Vec<SportVisionProduct>>> = vec![];

    for page in 0..=last_page {
        let request = klijent.get(format!("https://sportvision.rs/proizvodi/page-{page}"));
        join_handles.push(task::spawn(get_products(request)));
    }
    for handle in join_handles {
        products.extend(handle.await.unwrap());
    }

    let mut file = fs::File::create("products.json").expect("Couldn't open/create products.json");
    file.write_all(serde_json::to_string_pretty(&products).unwrap().as_bytes())
        .expect("Couldn't write to products.json");
}

async fn get_products(request: RequestBuilder) -> Vec<SportVisionProduct> {
    let mut products: Vec<SportVisionProduct> = vec![];
    let document = Html::parse_document(&request.send().await.unwrap().text().await.unwrap());
    for div in document
        .select(&Selector::parse(".wrapper-gridthree-view.product-item .row .item-data").unwrap())
    {
        products.push(SportVisionProduct::from_div(div));
    }

    products
}
