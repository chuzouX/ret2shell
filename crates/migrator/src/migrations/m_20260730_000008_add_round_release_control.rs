use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20260730_000008_add_round_release_control"
  }
}

#[derive(DeriveIden)]
enum Tournament {
  Table,
  CurrentRoundId,
  RoundControlMode,
}

#[derive(DeriveIden)]
enum TournamentRound {
  Table,
  ReleaseAudience,
  ReleaseTiming,
  EndMode,
  ReleaseAt,
  StartedAt,
  EndedAt,
  ReleasedAt,
  ManuallyReleased,
  ManuallyEnded,
  ReleaseVersion,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Tournament::Table)
          .add_column_if_not_exists(ColumnDef::new(Tournament::CurrentRoundId).big_integer())
          .add_column_if_not_exists(
            ColumnDef::new(Tournament::RoundControlMode)
              .string_len(16)
              .not_null()
              .default("manual_assisted"),
          )
          .to_owned(),
      )
      .await?;
    manager
      .alter_table(
        Table::alter()
          .table(TournamentRound::Table)
          .add_column_if_not_exists(
            ColumnDef::new(TournamentRound::ReleaseAudience)
              .json_binary()
              .not_null()
              .default("[\"staff\"]"),
          )
          .add_column_if_not_exists(
            ColumnDef::new(TournamentRound::ReleaseTiming)
              .string_len(16)
              .not_null()
              .default("on_enter"),
          )
          .add_column_if_not_exists(
            ColumnDef::new(TournamentRound::EndMode)
              .string_len(16)
              .not_null()
              .default("on_next_round"),
          )
          .add_column_if_not_exists(
            ColumnDef::new(TournamentRound::ReleaseAt).timestamp_with_time_zone(),
          )
          .add_column_if_not_exists(
            ColumnDef::new(TournamentRound::StartedAt).timestamp_with_time_zone(),
          )
          .add_column_if_not_exists(
            ColumnDef::new(TournamentRound::EndedAt).timestamp_with_time_zone(),
          )
          .add_column_if_not_exists(
            ColumnDef::new(TournamentRound::ReleasedAt).timestamp_with_time_zone(),
          )
          .add_column_if_not_exists(
            ColumnDef::new(TournamentRound::ManuallyReleased)
              .boolean()
              .not_null()
              .default(false),
          )
          .add_column_if_not_exists(
            ColumnDef::new(TournamentRound::ManuallyEnded)
              .boolean()
              .not_null()
              .default(false),
          )
          .add_column_if_not_exists(
            ColumnDef::new(TournamentRound::ReleaseVersion)
              .big_integer()
              .not_null()
              .default(0),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(TournamentRound::Table)
          .drop_column(TournamentRound::ReleaseVersion)
          .drop_column(TournamentRound::ManuallyEnded)
          .drop_column(TournamentRound::ManuallyReleased)
          .drop_column(TournamentRound::ReleasedAt)
          .drop_column(TournamentRound::EndedAt)
          .drop_column(TournamentRound::StartedAt)
          .drop_column(TournamentRound::ReleaseAt)
          .drop_column(TournamentRound::EndMode)
          .drop_column(TournamentRound::ReleaseTiming)
          .drop_column(TournamentRound::ReleaseAudience)
          .to_owned(),
      )
      .await?;
    manager
      .alter_table(
        Table::alter()
          .table(Tournament::Table)
          .drop_column(Tournament::RoundControlMode)
          .drop_column(Tournament::CurrentRoundId)
          .to_owned(),
      )
      .await
  }
}
