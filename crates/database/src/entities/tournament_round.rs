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
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
