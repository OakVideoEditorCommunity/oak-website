pub mod docs;
pub mod github;
pub mod r2;

pub use docs::DocsIndex;
pub use github::GithubClient;
pub use r2::{create_s3_client, R2Service};
