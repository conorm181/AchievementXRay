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

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/SearchGames/:url", get(get_steam_apps));
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_steam_apps(Path(UrlParams {url}): Path<UrlParams>) -> impl IntoResponse {
    // let _client = Client::new();
    let resp: Response = REQWEST_CLIENT.get("https://api.steampowered.com/ISteamApps/GetAppList/v0002/").send().await.unwrap();
    let status: StatusCode = resp.status();
    let json_response: Value = resp.json::<serde_json::Value>().await.unwrap();
    let _response: &Value = json_response.get("applist").and_then(|val| val.get("apps")).unwrap();
    // let search_term = "Balatro";
    let vals:Vec<String> = search(_response.clone(), url);
    // (status, Json(_response.clone()))
    (status, Json(vals))
}

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

