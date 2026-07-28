use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chat")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub tournament_id: i64,
  pub user_id: i64,
  #[sea_orm(column_type = "Text")]
  pub content: String,
  pub is_staff: bool,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::tournament::Entity",
    from = "Column::TournamentId",
    to = "super::tournament::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Tournament,
  #[sea_orm(
    belongs_to = "super::user::Entity",
    from = "Column::UserId",
    to = "super::user::Column::Id",
    on_update = "Cascade",
    on_delete = "Restrict"
  )]
  User,
}

impl Related<super::tournament::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Tournament.def()
  }
}

impl Related<super::user::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::User.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
