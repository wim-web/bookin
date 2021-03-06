pub mod twolo {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use reqwest::{Client, Result};
    use serde::{Deserialize, Serialize};

    pub type AccessToken = String;

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        expires_in: u32,
        token_type: String,
    }

    pub async fn access_token(c: Credentials) -> Result<AccessToken> {
        let url = format!(
            "{}?grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer&assertion={}",
            c.claim.aud,
            c.jwt()
        );

        let res = Client::new()
            .post(url)
            .header("Content-Length", 0)
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        Ok(res.access_token)
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Claim {
        aud: String,
        exp: usize,
        iat: usize,
        iss: String,
        scope: String,
    }

    pub struct Credentials {
        claim: Claim,
        private_key: String,
    }

    impl Credentials {
        pub fn from_service_account_json(json: String, scope: String) -> Self {
            let k = ServiceAccountKey::from_json(json);

            let iat = Utc::now();
            let exp = iat + Duration::minutes(30);

            let claim = Claim {
                aud: k.token_uri,
                iss: k.client_email,
                scope,
                exp: exp.timestamp() as usize,
                iat: iat.timestamp() as usize,
            };

            Credentials {
                claim,
                private_key: k.private_key,
            }
        }

        fn jwt(&self) -> String {
            let jwt = encode(
                &Header::new(jsonwebtoken::Algorithm::RS256),
                &self.claim,
                &EncodingKey::from_rsa_pem(self.private_key.as_bytes()).unwrap(),
            )
            .unwrap();
            jwt
        }
    }

    #[derive(Deserialize)]
    struct ServiceAccountKey {
        r#type: String,
        project_id: String,
        private_key_id: String,
        private_key: String,
        client_email: String,
        client_id: String,
        auth_uri: String,
        token_uri: String,
        auth_provider_x509_cert_url: String,
        client_x509_cert_url: String,
    }

    impl ServiceAccountKey {
        fn from_json(json: String) -> Self {
            let k: ServiceAccountKey = serde_json::from_str(json.as_str()).unwrap();
            k
        }
    }

    #[cfg(test)]
    mod tests {
        use std::{fs, str::FromStr};

        use super::*;

        fn json() -> String {
            fs::read_to_string("./test/test.json").unwrap()
        }

        #[test]
        fn service_account_key_from_json() {
            let k = ServiceAccountKey::from_json(String::from_str(&json()).unwrap());

            assert_eq!(k.r#type, "type");
            assert_eq!(k.project_id, "project_id");
            assert_eq!(k.client_email, "client_email");
            assert_eq!(k.client_id, "client_id");
            assert_eq!(k.auth_uri, "auth_uri");
            assert_eq!(k.token_uri, "token_uri");
            assert_eq!(k.auth_provider_x509_cert_url, "auth_provider_x509_cert_url");
            assert_eq!(k.client_x509_cert_url, "client_x509_cert_url");
        }

        #[test]
        fn two_legged_oauth_credentials_from_json() {
            let c = Credentials::from_service_account_json(
                String::from_str(&json()).unwrap(),
                "scope".to_string(),
            );

            assert_eq!(c.claim.aud, "token_uri");
            assert_eq!(c.claim.iss, "client_email");
            assert_eq!(c.claim.scope, "scope");
        }

        #[test]
        fn credentials_jwt() {
            let c = Credentials::from_service_account_json(
                String::from_str(&json()).unwrap(),
                "scope".to_string(),
            );

            assert_ne!("", c.jwt());
        }
    }
}

pub mod drive {
    use reqwest::Client as reqwest_c;
    use reqwest::Result;
    use serde::Deserialize;

    pub struct Client {
        token: String,
        base_uri: String,
        client: reqwest_c,
    }

    impl Client {
        pub fn new(access_token: String) -> Self {
            Self {
                token: access_token,
                base_uri: "https://www.googleapis.com/drive/v3".to_string(),
                client: reqwest_c::new(),
            }
        }

        pub async fn files(&self, page_token: String, query: String) -> Result<FilesResponse> {
            let res = self
                .client
                .get(format!("{}/files", self.base_uri))
                .query(&[
                    ("access_token", &self.token),
                    (
                        "fields",
                        &"kind,nextPageToken,incompleteSearch,files(id,kind,name,webContentLink,webViewLink,thumbnailLink,mimeType,createdTime,modifiedTime,fileExtension)".to_string(),
                    ),
                    ("q", &query),
                    ("pageToken", &page_token)
                ])
                .send()
                .await?
                .json::<FilesResponse>()
                .await?;

            Ok(res)
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct FilesResponse {
        kind: String,
        #[serde(rename = "incompleteSearch")]
        incomplete_search: bool,
        #[serde(rename = "nextPageToken")]
        pub next_page_token: Option<String>,
        pub files: Files,
    }

    pub type Files = Vec<File>;

    #[derive(Debug, Deserialize)]
    pub struct File {
        pub kind: String,
        pub id: String,
        pub name: String,
        #[serde(rename = "webContentLink")]
        pub download_link: Option<String>,
        #[serde(rename = "webViewLink")]
        pub link: String,
        #[serde(rename = "thumbnailLink")]
        pub thumbnail_link: Option<String>,
        #[serde(rename = "mimeType")]
        pub mime_type: String,
        #[serde(rename = "createdTime")]
        pub created_time: String,
        #[serde(rename = "modifiedTime")]
        pub modified_time: String,
        #[serde(rename = "fileExtension")]
        pub file_extension: Option<String>,
    }
}
