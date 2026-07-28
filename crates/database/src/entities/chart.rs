use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chart")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub tournament_id: i64,
  pub round_id: i64,
  pub tag_id: i64,
  pub title: String,
  pub artist: String,
  pub charter: String,
  pub difficulty: String,
  pub level_constant: f64,
  pub cover: Option<String>,
  pub order_index: i32,
  pub weight_millionths: i64,
  #[sea_orm(column_type = "JsonBinary")]
  pub metadata: Json,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
