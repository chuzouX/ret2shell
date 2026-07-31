use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20260731_000002_add_tournament_rules_and_announcements_visibility"
  }
}

#[derive(DeriveIden)]
enum Tournament {
  Table,
  Rules,
  RulesVisible,
  AnnouncementsVisible,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Tournament::Table)
          .add_column_if_not_exists(ColumnDef::new(Tournament::Rules).text())
          .add_column_if_not_exists(
            ColumnDef::new(Tournament::RulesVisible)
              .boolean()
              .not_null()
              .default(false),
          )
          .add_column_if_not_exists(
            ColumnDef::new(Tournament::AnnouncementsVisible)
              .boolean()
              .not_null()
              .default(false),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Tournament::Table)
          .drop_column(Tournament::AnnouncementsVisible)
          .drop_column(Tournament::RulesVisible)
          .drop_column(Tournament::Rules)
          .to_owned(),
      )
      .await
  }
}
