use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Releases::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Releases::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Releases::Version).string().not_null().unique_key())
                    .col(ColumnDef::new(Releases::TagName).string().not_null())
                    .col(ColumnDef::new(Releases::ReleaseNotes).text())
                    .col(ColumnDef::new(Releases::IsPrerelease).boolean().not_null().default(false))
                    .col(ColumnDef::new(Releases::PublishedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Releases::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Releases::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ReleaseAssets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ReleaseAssets::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ReleaseAssets::ReleaseId).uuid().not_null())
                    .col(ColumnDef::new(ReleaseAssets::Platform).string_len(32).not_null())
                    .col(ColumnDef::new(ReleaseAssets::Arch).string_len(32))
                    .col(ColumnDef::new(ReleaseAssets::Filename).string().not_null())
                    .col(ColumnDef::new(ReleaseAssets::GithubUrl).string().not_null())
                    .col(ColumnDef::new(ReleaseAssets::R2Key).string())
                    .col(ColumnDef::new(ReleaseAssets::R2Etag).string())
                    .col(ColumnDef::new(ReleaseAssets::SizeBytes).big_integer())
                    .col(ColumnDef::new(ReleaseAssets::SyncStatus).string_len(32).not_null().default("pending"))
                    .col(ColumnDef::new(ReleaseAssets::SyncedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ReleaseAssets::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(ReleaseAssets::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_release_assets_release_id")
                            .from(ReleaseAssets::Table, ReleaseAssets::ReleaseId)
                            .to(Releases::Table, Releases::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DownloadLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DownloadLogs::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DownloadLogs::AssetId).uuid().not_null())
                    .col(ColumnDef::new(DownloadLogs::Ip).string_len(64))
                    .col(ColumnDef::new(DownloadLogs::UserAgent).text())
                    .col(ColumnDef::new(DownloadLogs::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_download_logs_asset_id")
                            .from(DownloadLogs::Table, DownloadLogs::AssetId)
                            .to(ReleaseAssets::Table, ReleaseAssets::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DownloadLogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ReleaseAssets::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Releases::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Releases {
    Table,
    Id,
    Version,
    TagName,
    ReleaseNotes,
    IsPrerelease,
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ReleaseAssets {
    Table,
    Id,
    ReleaseId,
    Platform,
    Arch,
    Filename,
    GithubUrl,
    R2Key,
    R2Etag,
    SizeBytes,
    SyncStatus,
    SyncedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum DownloadLogs {
    Table,
    Id,
    AssetId,
    Ip,
    UserAgent,
    CreatedAt,
}
