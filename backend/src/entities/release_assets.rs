use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "release_assets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub release_id: Uuid,
    pub platform: String,
    pub arch: Option<String>,
    pub filename: String,
    pub github_url: String,
    pub r2_key: Option<String>,
    pub r2_etag: Option<String>,
    pub size_bytes: Option<i64>,
    pub sync_status: String,
    pub synced_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::releases::Entity",
        from = "Column::ReleaseId",
        to = "super::releases::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Releases,
    #[sea_orm(has_many = "super::download_logs::Entity")]
    DownloadLogs,
}

impl Related<super::releases::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Releases.def()
    }
}

impl Related<super::download_logs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DownloadLogs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
