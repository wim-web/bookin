use bookin::{
    google::{drive, twolo},
    notion,
};
use std::{
    collections::HashMap,
    fs,
    time::{self, Duration},
};

#[tokio::main]
async fn main() {
    let json = fs::read_to_string("./notion.json").unwrap();
    let json: HashMap<String, String> = serde_json::from_str(&json).unwrap();

    let notion = notion::Client::new(json["secret"].clone());

    let latest = notion
        .get_database(
            "2864810e503441749bc2a129ef2591e4".to_string(),
            r#"{"sorts":[{"timestamp":"created_time","direction":"descending"}],"page_size":1}"#
                .to_string(),
        )
        .await
        .unwrap();

    let json = fs::read_to_string("./credentials.json").unwrap();
    let credentials = twolo::Credentials::from_service_account_json(
        json,
        "https://www.googleapis.com/auth/drive".to_string(),
    );
    let access_token = twolo::access_token(credentials).await.unwrap();
    let google_drive = drive::Client::new(access_token);

    let mut files: Vec<drive::File> = vec![];

    let pdf_folder_id = "1UuI8PxAJWn93e-Y9NqFK4MEFAJErkbod";

    let query = format!(
        "(mimeType=\"application/epub+zip\" or mimeType=\"application/pdf\") and modifiedTime > \"{}\" and '{}' in parents ",
        latest.results[0].created_time,
        pdf_folder_id
    );

    let mut r = google_drive
        .files("".to_string(), query.clone())
        .await
        .unwrap();

    files.append(&mut r.files);

    while let Some(token) = r.next_page_token {
        r = google_drive.files(token, query.clone()).await.unwrap();
        files.append(&mut r.files);
    }

    for f in files {
        notion
            .store_database(&f, "2864810e503441749bc2a129ef2591e4".to_string())
            .await
            .unwrap();
        // notion apiは 3res/s 程度の制限があるので
        std::thread::sleep(Duration::from_millis(100));
    }
}
