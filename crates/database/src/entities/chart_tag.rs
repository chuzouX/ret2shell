use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chart_tag")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub tournament_id: i64,
  pub round_id: i64,
  pub name: String,
  pub order_index: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
