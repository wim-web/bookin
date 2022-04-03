use bookin::google::{drive, twolo};
use std::fs;

#[tokio::main]
async fn main() {
    let json = fs::read_to_string("./credentials.json").unwrap();
    let credentials = twolo::Credentials::from_service_account_json(
        json,
        "https://www.googleapis.com/auth/drive".to_string(),
    );

    let access_token = twolo::access_token(credentials).await.unwrap();

    let client = drive::Client::new(access_token);

    let mut files: Vec<drive::File> = vec![];

    let mut r = client.files("".to_string()).await.unwrap();

    files.append(&mut r.files);

    while let Some(token) = r.next_page_token {
        r = client.files(token).await.unwrap();
        files.append(&mut r.files);
    }

    println!("{:?}", files.len());
}
