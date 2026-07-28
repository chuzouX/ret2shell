use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
  Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(16))")]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardKind {
  #[default]
  #[sea_orm(string_value = "individual")]
  Individual,
  #[sea_orm(string_value = "team")]
  Team,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "leaderboard_snapshot")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub tournament_id: i64,
  pub script_version_id: Option<i64>,
  pub kind: LeaderboardKind,
  #[sea_orm(column_type = "JsonBinary")]
  pub entries: Json,
  pub stale: bool,
  pub error: Option<String>,
  pub public_snapshot: bool,
  #[serde(with = "ts_seconds")]
  pub computed_at: DateTime<Utc>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
