use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tournament_chart_library")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub tournament_id: i64,
  pub chart_library_id: i64,
  pub round_id: i64,
  pub tag_id: i64,
  pub order_index: i32,
  pub weight_millionths: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
