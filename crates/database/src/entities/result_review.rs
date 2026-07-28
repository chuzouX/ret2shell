use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "result_review")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub result_id: i64,
  pub reviewer_id: i64,
  pub from_status: super::result::ResultStatus,
  pub to_status: super::result::ResultStatus,
  #[sea_orm(column_type = "Text", nullable)]
  pub reason: Option<String>,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
