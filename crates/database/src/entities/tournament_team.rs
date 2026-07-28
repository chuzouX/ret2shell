use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tournament_team")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub tournament_id: i64,
  pub name: String,
  pub captain_registration_id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
