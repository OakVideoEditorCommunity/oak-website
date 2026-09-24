use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bug_reports")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    /// The Oak version the reporter was running.
    pub app_version: String,
    pub content: String,
    pub email: Option<String>,
    /// R2 object key of the uploaded screenshot, if any.
    pub screenshot_key: Option<String>,
    pub screenshot_filename: Option<String>,
    /// R2 object key of the uploaded log file, if any.
    pub log_key: Option<String>,
    pub log_filename: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
