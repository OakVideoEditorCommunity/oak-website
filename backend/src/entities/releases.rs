use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "releases")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub version: String,
    pub tag_name: String,
    pub release_notes: Option<String>,
    pub is_prerelease: bool,
    pub published_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::release_assets::Entity")]
    ReleaseAssets,
}

impl Related<super::release_assets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReleaseAssets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
