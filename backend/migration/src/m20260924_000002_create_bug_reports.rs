use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BugReports::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BugReports::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BugReports::Title).string().not_null())
                    // The Oak version the reporter was running.
                    .col(ColumnDef::new(BugReports::AppVersion).string().not_null())
                    .col(ColumnDef::new(BugReports::Content).text().not_null())
                    .col(ColumnDef::new(BugReports::Email).string())
                    // R2 object keys + original filenames for the attachments.
                    .col(ColumnDef::new(BugReports::ScreenshotKey).string())
                    .col(ColumnDef::new(BugReports::ScreenshotFilename).string())
                    .col(ColumnDef::new(BugReports::LogKey).string())
                    .col(ColumnDef::new(BugReports::LogFilename).string())
                    .col(ColumnDef::new(BugReports::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BugReports::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum BugReports {
    Table,
    Id,
    Title,
    AppVersion,
    Content,
    Email,
    ScreenshotKey,
    ScreenshotFilename,
    LogKey,
    LogFilename,
    CreatedAt,
}
