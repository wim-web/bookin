use reqwest::Method;
use reqwest::RequestBuilder;

use reqwest::header;
use reqwest::Client as reqwest_c;
use reqwest::Result;
use serde::Deserialize;

use crate::google::drive::File;

pub struct Client {
    base_uri: String,
    secret: String,
}

impl Client {
    pub fn new(secret: String) -> Self {
        Self {
            base_uri: "https://api.notion.com/v1".to_string(),
            secret,
        }
    }

    fn baseBuilder(&self, method: Method, url: String) -> RequestBuilder {
        reqwest_c::new()
            .request(method, url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.secret))
            .header(header::CONTENT_TYPE, "application/json")
            .header("Notion-Version", "2021-08-16")
    }

    pub async fn get_database(&self, id: String, data: String) -> Result<DatabaseResponse> {
        let res = self
            .baseBuilder(
                Method::POST,
                format!("{}/databases/{}/query", self.base_uri, id),
            )
            .body(data)
            .send()
            .await?
            .json::<DatabaseResponse>()
            .await?;

        Ok(res)
    }

    pub async fn store_database(&self, file: &File, database_id: String) -> Result<()> {
        let s = "https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcR62Ye2YTQqGErSkeAcj-0ZceMxg_10AltAkQ&usqp=CAU".to_string();
        let _res = self
            .baseBuilder(Method::POST, format!("{}/pages", self.base_uri))
            .body(format!(
                r#"
                {{
                    "parent": {{"database_id": "{}"}},
                    "properties": {{
                        "Name": {{
                            "title": [
                                {{"text": {{"content": "{}"}}}}
                            ]
                        }},
                        "id": {{
                            "rich_text": [
                                {{"text": {{"content": "{}"}}}}
                            ]
                        }},
                        "url": {{
                            "url": "{}"
                        }}
                    }},
                    "cover": {{
                        "type": "external",
                        "external": {{
                            "url": "{}"
                        }}
                    }}
                }}
            "#,
                database_id,
                file.name,
                file.id,
                file.link,
                match &file.thumbnail_link {
                    Some(link) => link,
                    None => &s,
                }
            ))
            .send()
            .await?;

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct DatabaseResponse {
    pub object: String,
    pub results: Vec<DatabaseResult>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseResult {
    pub object: String,
    pub id: String,
    pub created_time: String,
    pub last_edited_time: String,
}
