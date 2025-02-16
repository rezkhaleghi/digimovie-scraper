#[macro_use]
extern crate lazy_static;

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use mongodb::{bson::doc, options::ClientOptions, Client,bson::DateTime,bson};
use std::error::Error;
use tokio;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use dotenv::dotenv;
use std::env;


#[derive(Debug, Serialize, Deserialize)]
struct Movie {
    title: String,
    imdb_id: String,
    imdb_rating: String,
    duration: String,
    genres: Vec<String>,
    director: String,
    stars: Vec<String>,
    country: String,
    description: String,
    metacritic_score: String,
    awards: String,
    image_url: String,
    has_subtitle: bool,
    trailer_link: String,
    page_number: u32,  
    content_type : String,
    slug: Option<String>,
    source:String
}

#[derive(Debug, Serialize, Deserialize)]
struct DownloadLinks {
    imdb_id: String,
    slug: String,
    last_updated:DateTime,
    sections: Vec<DownloadSection>,
    source:String
}

#[derive(Debug, Serialize, Deserialize)]
struct DownloadSection {
    quality: String,
    size: String,
    sub_type: Option<String>,
    encoder: Option<String>,
    download_link: Option<String>,
}


lazy_static! {
    // env files
    static ref DM_COOKIE_NAME: String = env::var("DM_COOKIE_NAME").unwrap_or_else(|_| "your-digimoviez.com-cookie-name".to_string());
    static ref DM_COOKIE_VALUE: String = env::var("DM_COOKIE_VALUE").unwrap_or_else(|_| "your-digimoviez.com-cookie-value".to_string());
    static ref DM_COOKIE_EXPIRES: String = env::var("DM_COOKIE_EXPIRES").unwrap_or_else(|_| "2028-12-12T06:11:29.470Z".to_string());
    static ref DB_NAME: String = env::var("DB_NAME").unwrap_or_else(|_| "digimoviez_fetcher".to_string());
    // general movie data selectors
    static ref SLUG_SELECTOR: Selector = Selector::parse(".title_h h2.lato_font a").unwrap();
    static ref MOVIES_SELECTOR: Selector = Selector::parse(".item_def_loop").unwrap();
    static ref TITLE_SELECTOR: Selector = Selector::parse("h2.lato_font a").unwrap();
    static ref IMDB_RATING_SELECTOR: Selector = Selector::parse(".imdb_rate_holder .rate_num strong").unwrap();
    static ref DURATION_SELECTOR: Selector = Selector::parse(".meta_item ul li:nth-child(2) .res_item").unwrap();
    static ref GENRE_SELECTOR: Selector = Selector::parse("li:nth-child(3) .res_item a").unwrap();
    static ref DIRECTOR_SELECTOR: Selector = Selector::parse("li:nth-child(4) .res_item a").unwrap();
    static ref STARS_SELECTOR: Selector = Selector::parse("li:nth-child(5) .res_item a").unwrap();
    static ref COUNTRY_SELECTOR: Selector = Selector::parse("li:nth-child(6) .res_item a").unwrap();
    static ref DESCRIPTION_SELECTOR: Selector = Selector::parse(".plot_text").unwrap();
    static ref METACRITIC_SELECTOR: Selector = Selector::parse(".greenlab").unwrap();
    static ref AWARDS_SELECTOR: Selector = Selector::parse(".award_item .text_hover").unwrap();
    static ref IMAGE_SELECTOR: Selector = Selector::parse(".cover img").unwrap();
    static ref SUBTITLE_SELECTOR: Selector = Selector::parse(".subtitles_item").unwrap();
    static ref TRAILER_SELECTOR: Selector = Selector::parse(".show_trailer").unwrap();
    // movie links selector
    static ref DM_QUALITY_SELECTOR: Selector = Selector::parse(".side_left .head_left_side h3").unwrap();
    static ref DM_DOWNLOAD_SELECTOR: Selector = Selector::parse(".dllink_holder_ham .body_dllink_movies .itemdl").unwrap();
    static ref DM_ENCODER_SELECTOR: Selector = Selector::parse(".item_meta.encoder_dl").unwrap();
    static ref DM_SIZE_SELECTOR: Selector = Selector::parse(".item_meta.size_dl").unwrap();
    static ref DM_LINK_SELECTOR: Selector = Selector::parse(".btn_row.btn_dl").unwrap();
    static ref DM_TITLE_LINK_SELECTOR: Selector = Selector::parse("div.title_h h2 a").unwrap();
    // series links selector
    static ref DM_SERIES_SEASON_SELECTOR: Selector = Selector::parse(".item_row_series.parent_item").unwrap();
    static ref DM_SERIES_QUALITY_SELECTOR: Selector = Selector::parse(".head_left_side h3").unwrap();
    static ref DM_SERIES_EPISODE_SELECTOR: Selector = Selector::parse(".part_item.online_btn .partlink").unwrap();
    static ref DM_SERIES_SIZE_SELECTOR: Selector = Selector::parse(".item_meta .size_dl").unwrap();
    // general series data selectors
    static ref DM_SERIES_SLUG_SELECTOR: Selector = Selector::parse(".title_h h2.lato_font a").unwrap();
    static ref DM_SERIES_TITLE_SELECTOR: Selector = Selector::parse("h2.lato_font a").unwrap();
    static ref DM_SERIES_IMAGE_SELECTOR: Selector = Selector::parse(".cover img").unwrap();
}


async fn fetch_document(
    client: &reqwest::Client,
    url: &str,
) -> Result<Html, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    let cookie_string = format!(
        "{}={}; Domain=digimoviez.com; Path=/; Expires={};",
        *DM_COOKIE_NAME, *DM_COOKIE_VALUE, *DM_COOKIE_EXPIRES
    );
    headers.insert(COOKIE, HeaderValue::from_str(&cookie_string).unwrap());

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await?;
    let body = response
        .text()
        .await?;
    Ok(Html::parse_document(&body))
}


async fn fetch_movie_links(
    client: &reqwest::Client,
    slug: &str,
) -> Result<Vec<DownloadSection>, Box<dyn Error>> {
    let url = format!("https://digimoviez.com/{}", slug);
    
    let document = fetch_document(client, &url).await?;
    
    let mut sections = Vec::new();

    
    for dl_section in document.select(&DM_DOWNLOAD_SELECTOR) {
        let quality = dl_section
        .select(&DM_QUALITY_SELECTOR)
        .next()
        .map(|e| e.inner_html().trim().to_string())
        .unwrap_or_default();

    let encoder = dl_section
        .select(&DM_ENCODER_SELECTOR)
        .next()
        .map(|e| e.inner_html().trim().replace("Encoder : ", ""))
        .unwrap_or_default();

    let size = dl_section
        .select(&DM_SIZE_SELECTOR)
        .next()
        .map(|e| e.inner_html().trim().to_string())
        .unwrap_or_default();

        let download_link = dl_section
        .select(&DM_LINK_SELECTOR)
        .next()
        .and_then(|e| e.value().attr("href"))
        .map(|link| clean_download_link(link))  // it removes md5 hash from download link
        .unwrap_or_default();



        if !download_link.is_empty() {
            sections.push(DownloadSection {
                quality,
                size,
                sub_type: None,
                encoder: Some(encoder),
                download_link: Some(download_link),
            });
        }
            

    }

    
    Ok(sections)
}

async fn fetch_movies(page_number: u32, client: &reqwest::Client, mongo_client: &Client) -> Result<Vec<Movie>, Box<dyn Error>> {
    let url = format!("https://digimoviez.com/page/{}/", page_number);

    let response = client.get(&url).send().await?.text().await?;
    let document = Html::parse_document(&response);
    
    let mut movies = Vec::new();

    for element in document.select(&MOVIES_SELECTOR) {
        let slug = element
            .select(&SLUG_SELECTOR)
            .next()
            .and_then(|e| e.value().attr("href"))
            .map(|href| href
                .trim_start_matches("https://digimoviez.com/")
                .trim_end_matches('/')
                .to_string())
            .unwrap_or_default();

        // Get IMDB ID separately from the IMDB rating link
        let title_element = element.select(&TITLE_SELECTOR).next().unwrap();
        let href = title_element.value().attr("href").unwrap_or("");
        let imdb_id = href.split('/').filter(|s| s.starts_with("tt")).next().unwrap_or("").to_string();


        let movie = Movie {
            source:"DigiMovie".to_string(),
            content_type: if title_element.text().collect::<String>().contains("فیلم") {
                "Movie".to_string()
            } else if title_element.text().collect::<String>().contains("انیمیشن") {
                "Animation".to_string()
            } else {
                "Unknown".to_string()
            },
            title: title_element.text().collect::<String>(),
            imdb_id: imdb_id.clone(),
            slug:Some(slug),  // Add the extracted slug
            imdb_rating: element.select(&IMDB_RATING_SELECTOR).next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default(),
            duration: element.select(&DURATION_SELECTOR).next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default(),
            genres: element.select(&GENRE_SELECTOR)
                .map(|e| e.text().collect::<String>())
                .collect(),
            director: element.select(&DIRECTOR_SELECTOR).next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default(),
            stars: element.select(&STARS_SELECTOR)
                .map(|e| e.text().collect::<String>())
                .collect(),
            country: element.select(&COUNTRY_SELECTOR).next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default(),
            description: element.select(&DESCRIPTION_SELECTOR).next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default(),
            metacritic_score: element.select(&METACRITIC_SELECTOR).next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default(),
            awards: element.select(&AWARDS_SELECTOR).next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default(),
            image_url: element.select(&IMAGE_SELECTOR).next()
                .and_then(|e| e.value().attr("src"))
                .unwrap_or("")
                .to_string(),
            has_subtitle: element.select(&SUBTITLE_SELECTOR).next().is_some(),
            trailer_link: element.select(&TRAILER_SELECTOR).next()
                .and_then(|e| e.value().attr("data-trailerlink"))
                .unwrap_or("")
                .to_string(),
            page_number,
        };

        let slug_clone = movie.slug.clone().unwrap();
        let imdb_clone = movie.imdb_id.clone();
        match fetch_movie_links(client, &slug_clone).await {
            Ok(sections) => {
                if !sections.is_empty() {
                    if let Err(e) = save_download_links_to_mongodb(mongo_client, &slug_clone, sections,&imdb_clone).await {
                        println!("Error saving download links for {}: {:?}", slug_clone, e);
                    }
                }
            },
            Err(e) => {
                println!("Error fetching download links for {}: {:?}", slug_clone, e);
            }
        }

        movies.push(movie);
    }

    println!("Found {} movies on page {}", movies.len(), page_number);
    Ok(movies)
}


fn clean_download_link(url: &str) -> String {
    if let Some(question_mark_index) = url.find('?') {
        url[..question_mark_index].to_string()
    } else {
        url.to_string()
    }
}

async fn get_mongo_client() -> Result<Client, Box<dyn Error>> {
    dotenv().ok(); // Load environment variables from a .env file, if it exists
    let client_uri = env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let options = ClientOptions::parse(&client_uri).await?;
    Ok(Client::with_options(options)?)
}

async fn save_movies_to_mongodb(client: &Client, movies: Vec<Movie>) -> mongodb::error::Result<()> {
    let db = client.database(&DB_NAME);
    let collection = db.collection::<Movie>("movies");
    
    println!("Attempting to save {} movies to database", movies.len());

    for movie in movies {
        let result = collection.update_one(
            doc! { "imdb_id": &movie.imdb_id },
            doc! { "$set": bson::to_document(&movie).unwrap() },
            mongodb::options::UpdateOptions::builder().upsert(true).build(),
        ).await?;

        println!("Movie {} {}", 
            movie.imdb_id,
            if result.upserted_id.is_some() { "inserted" } else { "updated" }
        );
    }

    let total_count = collection.count_documents(None, None).await?;
    println!("Total movies in database: {}", total_count);
    
    Ok(())
}

async fn save_download_links_to_mongodb(client: &Client, slug: &str, sections: Vec<DownloadSection>, imdb_id : &str) -> mongodb::error::Result<()> {
    let db = client.database(&DB_NAME);
    let collection = db.collection::<DownloadLinks>("download_links");

    println!("Attempting to save download links for {} to database", slug);
    
    let download_links = DownloadLinks {
        slug: slug.to_string(),
        imdb_id: imdb_id.to_string(),
        last_updated: DateTime::now(),
        sections,
        source:"DigiMovie".to_string()
    };

    let result = collection.update_one(
        doc! { "imdb_id": imdb_id },
        doc! { "$set": bson::to_document(&download_links).unwrap() },
        mongodb::options::UpdateOptions::builder().upsert(true).build(),
    ).await?;

    println!("Download links for {} {}", 
        imdb_id,
        if result.upserted_id.is_some() { "inserted" } else { "updated" }
    );

    Ok(())
}

async fn get_last_scraped_page(client: &Client) -> mongodb::error::Result<u32> {
    let db = client.database(&DB_NAME);
    let collection = db.collection::<bson::Document>("progress");
    if let Some(doc) = collection.find_one(None, None).await? {
        if let Some(page) = doc.get_i32("last_page").ok() {
            return Ok(page as u32);
        }
    }
    Ok(889) // Start from last page if no progress found
}

async fn update_last_scraped_page(client: &Client, page: u32) -> mongodb::error::Result<()> {
    let db = client.database(&DB_NAME);
    let collection = db.collection::<bson::Document>("progress");
    collection.update_one(
        doc! {},
        doc! { "$set": { "last_page": page as i32 } },
        mongodb::options::UpdateOptions::builder().upsert(true).build(),
    ).await?;
    println!("Updated progress: page {}", page);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Try to connect to MongoDB
    println!("Connecting to MongoDB...");
    let mongo_client = match get_mongo_client().await {
        Ok(client) => {
            println!("Successfully connected to MongoDB!");
            client
        },
        Err(e) => {
            eprintln!("Failed to connect to MongoDB: {:?}", e);
            std::process::exit(1);
        }
    };

    // Test MongoDB connection by performing a simple operation
    if let Err(e) = mongo_client
        .database(&DB_NAME)
        .run_command(mongodb::bson::doc! { "ping": 1 }, None)
        .await 
    {
        eprintln!("MongoDB connection test failed: {:?}", e);
        std::process::exit(1);
    }

    // Initialize HTTP client
    println!("Initializing HTTP client...");
    let http_client = match reqwest::Client::builder().build() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to create HTTP client: {:?}", e);
            std::process::exit(1);
        }
    };

    // Get the last scraped page
    println!("Fetching last scraped page...");
    let mut page = match get_last_scraped_page(&mongo_client).await {
        Ok(p) => {
            println!("Resuming from page {}", p);
            p
        },
        Err(e) => {
            eprintln!("Failed to fetch last scraped page: {:?}", e);
            std::process::exit(1);
        }
    };

    println!("Starting scraping process...");
    let mut consecutive_errors = 0;
    const MAX_CONSECUTIVE_ERRORS: u32 = 3;

    while page > 0 {
        println!("\nProcessing page: {}", page);
        
        match fetch_movies(page, &http_client, &mongo_client).await {
            Ok(movies) => {
                consecutive_errors = 0;  // Reset error counter on success
                
                if !movies.is_empty() {
                    match save_movies_to_mongodb(&mongo_client, movies).await {
                        Ok(_) => {
                            if let Err(e) = update_last_scraped_page(&mongo_client, page - 1).await {
                                eprintln!("Failed to update progress for page {}: {:?}", page, e);
                            }
                            page -= 1;
                        },
                        Err(e) => {
                            eprintln!("Error saving movies from page {}: {:?}", page, e);
                            consecutive_errors += 1;
                        }
                    }
                } else {
                    println!("No movies found on page {}", page);
                    page -= 1;
                }
                
                println!("Waiting 1 second before next request...");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err(err) => {
                eprintln!("Error fetching page {}: {:?}", page, err);
                consecutive_errors += 1;
                
                println!("Waiting 5 seconds before retry...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }

        // Check for too many consecutive errors
        if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
            eprintln!("Too many consecutive errors ({}). Stopping scraper.", consecutive_errors);
            std::process::exit(1);
        }
    }

    println!("Scraping completed successfully!");
    Ok(())
}