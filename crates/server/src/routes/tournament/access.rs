use r2s_database::{
  tournament,
  tournament_staff::{self, StaffRole},
  user::Permission,
};
use r2s_migrator::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{middleware::auth::Token, traits::ResponseError};

pub async fn tournament(db: &Database, id: i64) -> Result<tournament::Model, ResponseError> {
  tournament::Entity::find_by_id(id)
    .one(&db.conn)
    .await?
    .ok_or_else(|| ResponseError::NotFound("tournament not found".to_owned()))
}

pub fn authenticated(token: &Token) -> Result<(), ResponseError> {
  if token.id <= 0 {
    Err(ResponseError::Unauthorized(
      "authentication required".to_owned(),
    ))
  } else {
    Ok(())
  }
}

/// Allows upload/review. Requires `ChartLibrary` or `DevOps` permission.
pub fn require_chart_manager(token: &Token) -> Result<(), ResponseError> {
  authenticated(token)?;
  if token.permissions.0.contains(&Permission::DevOps)
    || token.permissions.0.contains(&Permission::ChartLibrary)
  {
    Ok(())
  } else {
    Err(ResponseError::Forbidden(
      "chart library permission required".to_owned(),
    ))
  }
}

/// Allows update/delete. `DevOps` can modify any chart; `ChartLibrary` holders
/// can only modify charts they uploaded (`created_by == token.id`).
pub fn require_chart_modifier(token: &Token, created_by: i64) -> Result<(), ResponseError> {
  authenticated(token)?;
  if token.permissions.0.contains(&Permission::DevOps)
    || (token.permissions.0.contains(&Permission::ChartLibrary) && created_by == token.id)
  {
    Ok(())
  } else {
    Err(ResponseError::Forbidden(
      "can only modify your own charts".to_owned(),
    ))
  }
}

pub async fn role(
  db: &Database, tournament_id: i64, token: &Token,
) -> Result<Option<StaffRole>, ResponseError> {
  if token.id <= 0 {
    return Ok(None);
  }
  Ok(
    tournament_staff::Entity::find()
      .filter(tournament_staff::Column::TournamentId.eq(tournament_id))
      .filter(tournament_staff::Column::UserId.eq(token.id))
      .one(&db.conn)
      .await?
      .map(|staff| staff.role),
  )
}

pub async fn require_judge(
  db: &Database, tournament_id: i64, token: &Token,
) -> Result<StaffRole, ResponseError> {
  role(db, tournament_id, token)
    .await?
    .ok_or_else(|| ResponseError::Forbidden("tournament staff access required".to_owned()))
}

pub async fn require_organizer(
  db: &Database, tournament_id: i64, token: &Token,
) -> Result<StaffRole, ResponseError> {
  match require_judge(db, tournament_id, token).await? {
    role @ (StaffRole::Owner | StaffRole::Organizer) => Ok(role),
    StaffRole::Judge => Err(ResponseError::Forbidden(
      "organizer access required".to_owned(),
    )),
  }
}

pub async fn require_owner(
  db: &Database, tournament_id: i64, token: &Token,
) -> Result<(), ResponseError> {
  match require_judge(db, tournament_id, token).await? {
    StaffRole::Owner => Ok(()),
    _ => Err(ResponseError::Forbidden("owner access required".to_owned())),
  }
}
