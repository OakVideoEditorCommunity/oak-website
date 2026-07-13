use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "download_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub asset_id: Uuid,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::release_assets::Entity",
        from = "Column::AssetId",
        to = "super::release_assets::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    ReleaseAssets,
}

impl Related<super::release_assets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReleaseAssets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
