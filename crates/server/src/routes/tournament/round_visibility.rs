use r2s_database::{
  registration,
  tournament_round::{self, EndMode, ReleaseTiming},
  tournament_staff::StaffRole,
};
use r2s_migrator::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::Value;

use crate::{middleware::auth::Token, traits::ResponseError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Audience {
  Staff,
  ApprovedParticipant,
  Public,
}

pub async fn audience(
  db: &Database, tournament_id: i64, token: &Token,
) -> Result<Audience, ResponseError> {
  let is_staff = r2s_database::tournament_staff::Entity::find()
    .filter(r2s_database::tournament_staff::Column::TournamentId.eq(tournament_id))
    .filter(r2s_database::tournament_staff::Column::UserId.eq(token.id))
    .one(&db.conn)
    .await?
    .map(|row| {
      matches!(
        row.role,
        StaffRole::Owner | StaffRole::Organizer | StaffRole::Judge
      )
    })
    .unwrap_or(false);
  if is_staff {
    return Ok(Audience::Staff);
  }
  if token.id > 0
    && registration::Entity::find()
      .filter(registration::Column::TournamentId.eq(tournament_id))
      .filter(registration::Column::UserId.eq(token.id))
      .filter(registration::Column::Status.eq(registration::RegistrationStatus::Approved))
      .one(&db.conn)
      .await?
      .is_some()
  {
    return Ok(Audience::ApprovedParticipant);
  }
  Ok(Audience::Public)
}

pub fn is_visible(
  row: &tournament_round::Model, audience: Audience, current_round_id: Option<i64>,
) -> bool {
  if audience == Audience::Staff {
    return true;
  }
  let audience_values = row.release_audience.as_array().cloned().unwrap_or_default();
  let allowed = match audience {
    Audience::ApprovedParticipant => audience_values
      .iter()
      .any(|value| value == "participants" || value == "public"),
    Audience::Public => audience_values.iter().any(|value| value == "public"),
    Audience::Staff => true,
  };
  if !allowed {
    return false;
  }
  if row.manually_released {
    return true;
  }
  match row.release_timing {
    ReleaseTiming::Immediate => true,
    ReleaseTiming::OnEnter => row.started_at.is_some() || current_round_id == Some(row.id),
    ReleaseTiming::OnEnd => row.ended_at.is_some(),
  }
}

pub fn validate_audience(value: &Value) -> Result<(), ResponseError> {
  let Some(values) = value.as_array() else {
    return Err(ResponseError::BadRequest(
      "release audience must be an array".to_owned(),
    ));
  };
  if values.is_empty()
    || values
      .iter()
      .any(|item| !matches!(item.as_str(), Some("public" | "participants" | "staff")))
  {
    return Err(ResponseError::BadRequest(
      "invalid release audience".to_owned(),
    ));
  }
  Ok(())
}

pub fn validate_schedule(row: &tournament_round::Model) -> Result<(), ResponseError> {
  validate_schedule_values(row.start_at, row.end_at, row.release_at, row.end_mode)
}

pub fn validate_schedule_values(
  start_at: Option<chrono::DateTime<chrono::Utc>>, end_at: Option<chrono::DateTime<chrono::Utc>>,
  release_at: Option<chrono::DateTime<chrono::Utc>>, end_mode: EndMode,
) -> Result<(), ResponseError> {
  if end_mode == EndMode::AtTime && release_at.is_none() {
    return Err(ResponseError::BadRequest(
      "round end time is required".to_owned(),
    ));
  }
  if end_at.is_some_and(|end| start_at.is_some_and(|start| end < start)) {
    return Err(ResponseError::BadRequest(
      "round end time must not precede start time".to_owned(),
    ));
  }
  if end_mode == EndMode::AtTime
    && release_at.is_some_and(|release| start_at.is_some_and(|start| release < start))
  {
    return Err(ResponseError::BadRequest(
      "round release time must not precede start time".to_owned(),
    ));
  }
  Ok(())
}
