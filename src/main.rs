use bookin::google;
use std::fs;

#[tokio::main]
async fn main() {
    let json = fs::read_to_string("./credentials.json").unwrap();
    let credentials = google::twolo::Credentials::from_service_account_json(
        json,
        "https://www.googleapis.com/auth/drive".to_string(),
    );

    let access_token = google::twolo::access_token(credentials).await.unwrap();

    println!("{}", access_token);
}
