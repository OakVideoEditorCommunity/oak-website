use sea_orm_migration::prelude::*;

mod m20240713_000001_create_releases_and_assets;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240713_000001_create_releases_and_assets::Migration),
        ]
    }
}
