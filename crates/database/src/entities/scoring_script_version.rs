use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "scoring_script_version")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub tournament_id: i64,
  pub version: i32,
  pub name: String,
  pub template_key: String,
  #[sea_orm(column_type = "Text")]
  pub source: String,
  pub source_hash: String,
  pub created_by: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub active: bool,
  pub immutable: bool,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
