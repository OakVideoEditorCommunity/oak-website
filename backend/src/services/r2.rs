use aws_config::SdkConfig;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::{Builder, Region},
    presigning::PresigningConfig,
    Client as S3Client,
};
use std::time::Duration;

use crate::config::R2Config;
use crate::error::{AppError, AppResult};

/// Builds an S3 client configured for Cloudflare R2.
pub fn create_s3_client(config: &R2Config) -> AppResult<S3Client> {
    let credentials = SharedCredentialsProvider::new(Credentials::new(
        &config.access_key_id,
        &config.secret_access_key,
        None,
        None,
        "r2-credentials",
    ));

    let sdk_config = SdkConfig::builder()
        .region(Region::new(config.region.clone()))
        .credentials_provider(credentials)
        .endpoint_url(config.endpoint_url.clone())
        .build();

    let s3_config = Builder::from(&sdk_config)
        .force_path_style(true)
        .build();

    Ok(S3Client::from_conf(s3_config))
}

/// Service wrapper around the S3 client for release asset operations.
pub struct R2Service {
    client: S3Client,
    bucket: String,
    #[allow(dead_code)]
    public_domain: Option<String>,
}

impl R2Service {
    /// Creates a new R2 service from configuration.
    pub fn new(client: S3Client, config: &R2Config) -> Self {
        Self {
            client,
            bucket: config.bucket_name.clone(),
            public_domain: config.public_domain.clone(),
        }
    }

    /// Uploads a byte stream to R2 under the given key.
    /// Returns the ETag returned by R2, if any.
    pub async fn upload_streaming(
        &self,
        key: &str,
        content_length: i64,
        body: aws_sdk_s3::primitives::ByteStream,
    ) -> AppResult<Option<String>> {
        let resp = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_length(content_length)
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::External(format!("R2 upload failed: {}", e)))?;

        Ok(resp.e_tag)
    }

    /// Generates a time-limited pre-signed URL for downloading an object.
    pub async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> AppResult<String> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(expires_in)
            .build()
            .map_err(|e| AppError::Internal(format!("presign config: {}", e)))?;

        let url = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| AppError::External(format!("R2 presign failed: {}", e)))?;

        Ok(url.uri().to_string())
    }

    /// Returns a public CDN URL if a public domain is configured.
    #[allow(dead_code)]
    pub fn public_url(
        &self,
        key: &str,
    ) -> Option<String> {
        self.public_domain
            .as_ref()
            .map(|domain| format!("{}/{}", domain.trim_end_matches('/'), key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::primitives::ByteStream;
    use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};

    fn test_r2_config(endpoint: &str) -> R2Config {
        R2Config {
            endpoint_url: endpoint.to_string(),
            access_key_id: "test-key".to_string(),
            secret_access_key: "test-secret".to_string(),
            bucket_name: "test-bucket".to_string(),
            public_domain: Some("https://cdn.example.com".to_string()),
            region: "auto".to_string(),
        }
    }

    #[test]
    fn create_s3_client_succeeds_with_valid_config() {
        let config = test_r2_config("https://r2.example.com");
        let client = create_s3_client(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn public_url_builds_cdn_link() {
        let config = test_r2_config("https://r2.example.com");
        let client = create_s3_client(&config).unwrap();
        let service = R2Service::new(client, &config);
        assert_eq!(
            service.public_url("releases/file.exe"),
            Some("https://cdn.example.com/releases/file.exe".to_string())
        );
    }

    #[tokio::test]
    async fn generate_presigned_url_returns_signed_url() {
        let config = test_r2_config("https://r2.example.com");
        let client = create_s3_client(&config).unwrap();
        let service = R2Service::new(client, &config);

        let url = service
            .generate_presigned_url("releases/file.exe", Duration::from_secs(300))
            .await
            .expect("presign should succeed");

        assert!(url.contains("releases/file.exe"));
        assert!(url.starts_with("https://r2.example.com/"));
    }

    #[tokio::test]
    async fn upload_streaming_sends_put_object() {
        let mock_server = MockServer::start().await;
        let key = "releases/00000000-0000-0000-0000-000000000000/file.exe";
        Mock::given(method("PUT"))
            .and(path(format!("/test-bucket/{}", key)))
            .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"abc123\""))
            .mount(&mock_server)
            .await;

        let mut config = test_r2_config(&mock_server.uri());
        config.public_domain = None;
        let client = create_s3_client(&config).unwrap();
        let service = R2Service::new(client, &config);

        let payload = b"fake binary data".to_vec();
        let stream = ByteStream::from(payload.clone());
        let etag = service
            .upload_streaming(key, payload.len() as i64, stream)
            .await
            .expect("upload should succeed");

        assert_eq!(etag, Some("\"abc123\"".to_string()));
    }
}
