use serde::{
    Deserialize,
    // Serialize
};
use lazy_static::lazy_static;
use std::sync::Arc;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, Router},
    // Json,
};
use reqwest::{Client, Response};
use serde_json::Value;
use cached::proc_macro::cached;

// Static http client to be reused across the app
// Held in an Arc for thread safety
lazy_static!{
    static ref REQWEST_CLIENT: Arc<Client> = {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-api-key", "your_api_key".parse().unwrap());
        Arc::new(Client::builder()
        .default_headers(headers)
        .build()
        .unwrap())
    };
}

#[derive(Deserialize)]
struct UrlParams {
    url: String,
}


#[derive(Clone)]
struct StatusResponse {
    status: StatusCode,
    body: Value
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    // build our application with a route
    let app = Router::new()
        .route("/SearchGames/:url", get(search_steam_apps));
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// - GET ENDPOINT
/// - Params: Url path parameter for search
/// - Returns: Status Code and JSON list of Returned Apps
async fn search_steam_apps(Path(UrlParams {url}): Path<UrlParams>) -> impl IntoResponse {
    let StatusResponse {status, body:_response} = get_steam_apps().await;  
    let vals:Vec<String> = search(_response.clone(), url);
    (status, Json(vals))
}

// Function to fetch all registered steam apps
// Cached every hour 
#[cached(time = 3600)]
async fn get_steam_apps() -> StatusResponse {
    let resp: Response = REQWEST_CLIENT.get("https://api.steampowered.com/ISteamApps/GetAppList/v0002/").send().await.unwrap();
    let status: StatusCode = resp.status();
    let json_response: Value = resp.json::<serde_json::Value>().await.unwrap();
    let jso = json_response.get("applist").and_then(|val| val.get("apps")).unwrap().clone();
    // (status, jso)
    return StatusResponse {status, body:jso}
}

// Standard JSON search just reading lines
fn search(json_data: Value, search_line: String) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    if let Some(apps) = json_data.as_array() {
        for app in apps{
            if app["name"].to_string().contains(&search_line) {
                // println!("{} found with id {}", app["name"], app["appid"]);
                found.push(String::from(app["name"].as_str().unwrap()));
            }
        }
    }
    if found.len() == 0 {
        found.push("Nothing found".to_string());
    }
    found
}

// TODO
// Test out using gson for faster json searching

