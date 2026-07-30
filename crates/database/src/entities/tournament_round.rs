use chrono::{DateTime, Utc, serde::ts_seconds_option};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tournament_round")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub tournament_id: i64,
  pub name: String,
  #[sea_orm(column_type = "Text", nullable)]
  pub description: Option<String>,
  pub order_index: i32,
  #[serde(with = "ts_seconds_option")]
  pub start_at: Option<DateTime<Utc>>,
  #[serde(with = "ts_seconds_option")]
  pub end_at: Option<DateTime<Utc>>,
  #[sea_orm(column_type = "JsonBinary")]
  pub release_audience: Json,
  pub release_timing: ReleaseTiming,
  pub end_mode: EndMode,
  #[serde(with = "ts_seconds_option")]
  pub release_at: Option<DateTime<Utc>>,
  #[serde(with = "ts_seconds_option")]
  pub started_at: Option<DateTime<Utc>>,
  #[serde(with = "ts_seconds_option")]
  pub ended_at: Option<DateTime<Utc>>,
  #[serde(with = "ts_seconds_option")]
  pub released_at: Option<DateTime<Utc>>,
  pub manually_released: bool,
  pub manually_ended: bool,
  pub release_version: i64,
}

#[derive(
  Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(16))")]
#[serde(rename_all = "snake_case")]
pub enum ReleaseTiming {
  #[default]
  #[sea_orm(string_value = "on_enter")]
  OnEnter,
  #[sea_orm(string_value = "immediate")]
  Immediate,
  #[sea_orm(string_value = "on_end")]
  OnEnd,
}

#[derive(
  Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(16))")]
#[serde(rename_all = "snake_case")]
pub enum EndMode {
  #[default]
  #[sea_orm(string_value = "on_next_round")]
  OnNextRound,
  #[sea_orm(string_value = "at_time")]
  AtTime,
  #[sea_orm(string_value = "manual")]
  Manual,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
