use sea_orm_migration::prelude::*;

mod m20240713_000001_create_releases_and_assets;
mod m20260924_000002_create_bug_reports;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240713_000001_create_releases_and_assets::Migration),
            Box::new(m20260924_000002_create_bug_reports::Migration),
        ]
    }
}
