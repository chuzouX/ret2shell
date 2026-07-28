use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
  Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(16))")]
#[serde(rename_all = "snake_case")]
pub enum ResultStatus {
  #[default]
  #[sea_orm(string_value = "pending")]
  Pending,
  #[sea_orm(string_value = "approved")]
  Approved,
  #[sea_orm(string_value = "rejected")]
  Rejected,
  #[sea_orm(string_value = "voided")]
  Voided,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "result")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub tournament_id: i64,
  pub chart_id: i64,
  pub registration_id: i64,
  pub team_id_snapshot: Option<i64>,
  pub submitted_by: i64,
  pub score: i64,
  pub accuracy_millionths: i64,
  pub max_combo: i32,
  pub full_combo: bool,
  pub all_perfect: bool,
  #[sea_orm(column_type = "JsonBinary")]
  pub judgments: Json,
  #[sea_orm(column_type = "JsonBinary")]
  pub metrics: Json,
  #[serde(with = "ts_seconds")]
  pub played_at: DateTime<Utc>,
  pub evidence: Option<String>,
  pub status: ResultStatus,
  pub replaces_result_id: Option<i64>,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
