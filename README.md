# DigiMoviez Scraper

DigiMoviez Movie Scraper

A robust Rust-based web scraper designed to collect movie information and download links from DigiMoviez.com, storing the data in MongoDB.

## Overview

This scraper is built to systematically collect movie metadata, including titles, ratings, cast information, and download links. It features robust error handling, rate limiting, and progress tracking to ensure reliable data collection.

## Features

- Scrapes comprehensive movie metadata (title, IMDB rating, duration, genres, etc.)
- Collects download links with quality and size information
- Stores data in MongoDB with upsert functionality

## Prerequisites

- Rust
- MongoDB
- Environment variables configuration (.env)

## Required Environment Variables

```env
MONGO_URI=your_mongodb_connection_string(example: mongodb://localhost:27017)
DB_NAME=your_database_name(example: "digimoviez")
DM_COOKIE_NAME=your_cookie_name(your cookie name from digimoviez.com)
DM_COOKIE_VALUE=your_cookie_value(your cookie value from digimoviez.com)
DM_COOKIE_EXPIRES=your_cookie_expiration(your cookie value from digimoviez.com example:"2025-02-19T06:11:29.470Z")
```

## Data Structure

### Movie Schema

```rust
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
    content_type: String,
    slug: Option<String>,
    source: String
}
```

### Download Links Schema

```rust
struct DownloadLinks {
    imdb_id: String,
    slug: String,
    last_updated: DateTime,
    sections: Vec<DownloadSection>,
    source: String
}
```

## Authentication Setup

To run the scraper, you need valid authentication cookies from DigiMoviez.com. Follow these steps:

1. Log in to DigiMoviez.com with your account
2. Open browser Developer Tools:
   - Chrome/Edge: Press F12 or Ctrl+Shift+I
   - Firefox: Press F12 or Ctrl+Shift+I
   - Safari: Enable developer menu in Preferences → Advanced
3. Navigate to:
   - Chrome/Edge: Application → Cookies
   - Firefox: Storage → Cookies
   - Safari: Storage → Cookies
4. Find the "wordpress_logged_in" cookie
5. Extract the following information:
   ```
   Cookie Name example: wordpress_logged_in_d13b2bvd21d06301434df5f427acb040
   Cookie Value example: your-user-name-on-digi-i-think%7C1739974278%7C182Q7p5IpD7eQ8gDwqNEYdAk21wsXtPwLJcxlUb656v%7C0263e859b2eefcf214d19ce002445da249116a01b792dbc06bfa4cbd6e0325d8
   Cookie Expiration example: Thu, 20 Feb 2025 02:11:18 GMT
   ```

## Installation

1. Clone the repository:

```bash
git clone [repository-url]
```

2. Install dependencies:

```bash
cargo build
```

3. Set up environment variables in a `.env` file

4. Run the scraper:

```bash
cargo run
```

## How It Works

1. **Progress Tracking**: The scraper starts from the last scraped page (defaults to 889 if no progress is found)
2. **Movie Collection**:
   - Fetches movie metadata from each page
   - Extracts download links for each movie
3. **Data Storage**:
   - Stores movie data in the `movies` collection
   - Stores download links in the `download_links` collection, you can query on it by "imdb_id" or "slug"

## Features in Detail

### Rate Limiting

- 1-second delay between successful requests
- 5-second delay after errors

### Progress Tracking

- Stores last scraped page in MongoDB
- Enables resume functionality
- Updates progress after successful page processing

## MongoDB Collections

1. **movies**: Stores movie metadata
2. **download_links**: Stores download links and quality information
3. **progress**: Tracks scraping progress

## Dependencies

- `tokio`: Async runtime
- `reqwest`: HTTP client
- `mongodb`: MongoDB driver
- `scraper`: HTML parsing
- `serde`: Serialization/Deserialization
- `lazy_static`: Static initialization
- `dotenv`: Environment variable management

## Limitations

- Dependent on site structure stability
- Requires valid cookie credentials
- Sequential page processing
- Single-threaded operation

## Future Improvements

1. Implement parallel processing
2. Add proxy support
3. Enhance error recovery
4. Add data validation
5. Implement retry queues
6. Add metrics collection
7. Implement backup functionality

## Author

Created and maintained by "PocketJack (Rez Khaleghi)"

- GitHub: https://github.com/rezkhaleghi
- Email: rezaxkhaleghi@gmail.com

## Support

If you enjoy this app and would like to support my work:

- Patreon : https://patreon.com/PocketJack
  Your support helps me continue developing free and open-source stuff.
