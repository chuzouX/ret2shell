use chrono::{
  DateTime, Utc,
  serde::{ts_seconds, ts_seconds_option},
};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
  Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(24))")]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
  #[default]
  #[sea_orm(string_value = "draft")]
  Draft,
  #[sea_orm(string_value = "registration")]
  Registration,
  #[sea_orm(string_value = "running")]
  Running,
  #[sea_orm(string_value = "review")]
  Review,
  #[sea_orm(string_value = "finished")]
  Finished,
  #[sea_orm(string_value = "archived")]
  Archived,
}

impl Lifecycle {
  pub fn can_transition_to(self, next: Self) -> bool {
    matches!(
      (self, next),
      (Self::Draft, Self::Registration)
        | (Self::Registration, Self::Running)
        | (Self::Running, Self::Review)
        | (Self::Review, Self::Finished)
        | (Self::Finished, Self::Archived)
    )
  }

  pub fn locks_teams(self) -> bool {
    matches!(
      self,
      Self::Running | Self::Review | Self::Finished | Self::Archived
    )
  }
}

#[derive(
  Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(16))")]
#[serde(rename_all = "snake_case")]
pub enum CompetitionMode {
  #[default]
  #[sea_orm(string_value = "individual")]
  Individual,
  #[sea_orm(string_value = "team")]
  Team,
  #[sea_orm(string_value = "both")]
  Both,
}

#[derive(
  Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(16))")]
#[serde(rename_all = "snake_case")]
pub enum EvidencePolicy {
  #[default]
  #[sea_orm(string_value = "optional")]
  Optional,
  #[sea_orm(string_value = "required")]
  Required,
  #[sea_orm(string_value = "disabled")]
  Disabled,
}

#[derive(
  Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(16))")]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardVisibility {
  #[default]
  #[sea_orm(string_value = "live")]
  Live,
  #[sea_orm(string_value = "frozen")]
  Frozen,
  #[sea_orm(string_value = "after_end")]
  AfterEnd,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tournament")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub name: String,
  pub brief: String,
  #[sea_orm(column_type = "Text", nullable)]
  pub description: Option<String>,
  pub owner_id: i64,
  pub lifecycle: Lifecycle,
  pub competition_mode: CompetitionMode,
  pub evidence_policy: EvidencePolicy,
  pub leaderboard_visibility: LeaderboardVisibility,
  pub cover: Option<String>,
  pub team_size_min: i32,
  pub team_size_max: i32,
  #[serde(with = "ts_seconds_option")]
  pub registration_start_at: Option<DateTime<Utc>>,
  #[serde(with = "ts_seconds_option")]
  pub registration_end_at: Option<DateTime<Utc>>,
  #[serde(with = "ts_seconds_option")]
  pub start_at: Option<DateTime<Utc>>,
  #[serde(with = "ts_seconds_option")]
  pub end_at: Option<DateTime<Utc>>,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  #[serde(with = "ts_seconds")]
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
  use super::Lifecycle;

  #[test]
  fn lifecycle_only_moves_forward_one_step() {
    assert!(Lifecycle::Draft.can_transition_to(Lifecycle::Registration));
    assert!(Lifecycle::Review.can_transition_to(Lifecycle::Finished));
    assert!(!Lifecycle::Draft.can_transition_to(Lifecycle::Running));
    assert!(!Lifecycle::Finished.can_transition_to(Lifecycle::Review));
  }
}
