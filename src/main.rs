use std::{fs, io::Write, path::PathBuf};

use clap::Parser;
use rand::random;
use reqwest::{Client, RequestBuilder};
use scraper::{selectable::Selectable, ElementRef, Html, Selector};
use serde::Serialize;
use tokio::task::{self, JoinHandle};

#[derive(Parser)]
#[command(version, about = "web-scraping proizvoda sa sportvision.rs")]
struct Cli {
    /// Uzima prozivode od prve stranice do LAST_PAGE
    #[arg(short, long, value_name = "LAST_PAGE")]
    proizvodi: Option<u16>,

    /// Uzima odecu od prve stranice do LAST_PAGE
    #[arg(short = 'd', long, value_name = "LAST_PAGE")]
    odeca: Option<u16>,

    /// Uzima obucu od prve stranice do LAST_PAGE
    #[arg(short = 'b', long, value_name = "LAST_PAGE")]
    obuca: Option<u16>,

    /// Uzima opremu od prve stranice do LAST_PAGE
    #[arg(short, long, value_name = "LAST_PAGE")]
    oprema: Option<u16>,

    /// Fajl u kome se cuvaju prozivodi
    #[arg(value_name = "FILE")]
    file: PathBuf,
}

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
    discount: u8,
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
        let discount: u8 = if let Some(x) = div
            .select(&Selector::parse(".text-discount").unwrap())
            .next()
        {
            x.text().next().unwrap().trim().parse().unwrap_or_default()
        } else {
            0
        };
        SportVisionProduct {
            image_url,
            image_url_high_res,
            brand_name,
            title,
            short_description,
            current_price,
            id,
            stock,
            discount,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut products: Vec<SportVisionProduct> = vec![];
    let klijent = Client::new();
    let mut join_handles: Vec<JoinHandle<Vec<SportVisionProduct>>> = vec![];

    if let Some(last_page) = cli.proizvodi {
        for page in 0..last_page {
            let request = klijent.get(format!("https://sportvision.rs/proizvodi/page-{page}"));
            join_handles.push(task::spawn(get_products(request)));
        }
    }
    if let Some(last_page) = cli.odeca {
        for page in 0..last_page {
            let request = klijent.get(format!("https://sportvision.rs/odeca/page-{page}"));
            join_handles.push(task::spawn(get_products(request)));
        }
    }
    if let Some(last_page) = cli.obuca {
        for page in 0..last_page {
            let request = klijent.get(format!("https://sportvision.rs/obuca/page-{page}"));
            join_handles.push(task::spawn(get_products(request)));
        }
    }
    if let Some(last_page) = cli.oprema {
        for page in 0..last_page {
            let request = klijent.get(format!("https://sportvision.rs/oprema/page-{page}"));
            join_handles.push(task::spawn(get_products(request)));
        }
    }

    for handle in join_handles {
        products.extend(handle.await.unwrap());
    }

    let mut file = fs::File::create(cli.file).expect("Couldn't open json file for writing");
    file.write_all(serde_json::to_string_pretty(&products).unwrap().as_bytes())
        .expect("Couldn't write to json file");
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
