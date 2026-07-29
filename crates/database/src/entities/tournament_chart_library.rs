use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
  Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(16))")]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
  #[default]
  #[sea_orm(string_value = "private")]
  Private,
  #[sea_orm(string_value = "public")]
  Public,
  #[sea_orm(string_value = "after_archive")]
  AfterArchive,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tournament_chart_library")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub tournament_id: i64,
  pub chart_library_id: Option<i64>,
  pub visibility: Visibility,
  pub round_id: i64,
  pub tag_id: i64,
  pub order_index: i32,
  pub weight_millionths: i64,
  pub description: Option<String>,
  pub title: String,
  pub artist: String,
  pub charter: String,
  pub difficulty: String,
  pub level_constant: f64,
  pub cover: Option<String>,
  #[sea_orm(column_type = "JsonBinary")]
  pub metadata: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
