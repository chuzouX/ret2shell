use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20260729_000007_add_tournament_lifecycle_schedule"
  }
}

#[derive(DeriveIden)]
enum Tournament {
  Table,
  RegistrationSchedule,
  RegistrationAt,
  RunningSchedule,
  RunningAt,
  ReviewSchedule,
  ReviewAt,
  FinishedSchedule,
  FinishedAt,
  OrganizerCanEditArchived,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Tournament::Table)
          .add_column_if_not_exists(
            ColumnDef::new(Tournament::RegistrationSchedule)
              .string_len(16)
              .not_null()
              .default("manual"),
          )
          .add_column_if_not_exists(
            ColumnDef::new(Tournament::RegistrationAt).timestamp_with_time_zone(),
          )
          .add_column_if_not_exists(
            ColumnDef::new(Tournament::RunningSchedule)
              .string_len(16)
              .not_null()
              .default("manual"),
          )
          .add_column_if_not_exists(
            ColumnDef::new(Tournament::RunningAt).timestamp_with_time_zone(),
          )
          .add_column_if_not_exists(
            ColumnDef::new(Tournament::ReviewSchedule)
              .string_len(16)
              .not_null()
              .default("manual"),
          )
          .add_column_if_not_exists(ColumnDef::new(Tournament::ReviewAt).timestamp_with_time_zone())
          .add_column_if_not_exists(
            ColumnDef::new(Tournament::FinishedSchedule)
              .string_len(16)
              .not_null()
              .default("manual"),
          )
          .add_column_if_not_exists(
            ColumnDef::new(Tournament::FinishedAt).timestamp_with_time_zone(),
          )
          .add_column_if_not_exists(
            ColumnDef::new(Tournament::OrganizerCanEditArchived)
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
          .drop_column(Tournament::OrganizerCanEditArchived)
          .drop_column(Tournament::FinishedAt)
          .drop_column(Tournament::FinishedSchedule)
          .drop_column(Tournament::ReviewAt)
          .drop_column(Tournament::ReviewSchedule)
          .drop_column(Tournament::RunningAt)
          .drop_column(Tournament::RunningSchedule)
          .drop_column(Tournament::RegistrationAt)
          .drop_column(Tournament::RegistrationSchedule)
          .to_owned(),
      )
      .await
  }
}
