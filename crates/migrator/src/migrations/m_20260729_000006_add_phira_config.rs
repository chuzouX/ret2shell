use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20260729_000006_add_phira_config"
  }
}

#[derive(Iden)]
enum Config {
  Table,
  Phira,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Config::Table)
          .add_column_if_not_exists(ColumnDef::new(Config::Phira).json_binary())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Config::Table)
          .drop_column(Config::Phira)
          .to_owned(),
      )
      .await
  }
}
