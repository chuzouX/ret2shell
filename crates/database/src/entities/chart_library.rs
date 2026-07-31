use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
  Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(16))")]
#[serde(rename_all = "snake_case")]
pub enum Status {
  #[sea_orm(string_value = "pending")]
  Pending,
  #[default]
  #[sea_orm(string_value = "approved")]
  Approved,
  #[sea_orm(string_value = "rejected")]
  Rejected,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chart_library")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub source_id: i64,
  pub external_id: Option<String>,
  pub created_by: i64,
  pub title: String,
  pub artist: String,
  pub charter: String,
  pub difficulty: String,
  pub level_constant: f64,
  pub cover: Option<String>,
  #[sea_orm(column_type = "JsonBinary")]
  pub metadata: Json,
  pub status: Status,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  #[serde(with = "ts_seconds")]
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
