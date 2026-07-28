use axum::{
  Extension, Json,
  extract::{Path, State},
  response::IntoResponse,
};
use chrono::{DateTime, Utc};
use r2s_database::{
  chart, chart_tag,
  registration::{self, RegistrationStatus},
  team_member,
  tournament::{self, CompetitionMode, EvidencePolicy, LeaderboardVisibility, Lifecycle},
  tournament_round,
  tournament_staff::{self, StaffRole},
  tournament_team,
  user::Permission,
};
use r2s_migrator::Database;
use sea_orm::{
  ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
  QueryFilter, QueryOrder, TransactionTrait,
};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{access, scoring};
use crate::{middleware::auth::Token, traits::ResponseError};

#[derive(Deserialize)]
pub struct CreateTournament {
  name: String,
  #[serde(default)]
  brief: String,
  description: Option<String>,
  competition_mode: Option<CompetitionMode>,
  evidence_policy: Option<EvidencePolicy>,
  leaderboard_visibility: Option<LeaderboardVisibility>,
  cover: Option<String>,
  team_size_min: Option<i32>,
  team_size_max: Option<i32>,
  registration_start_at: Option<DateTime<Utc>>,
  registration_end_at: Option<DateTime<Utc>>,
  start_at: Option<DateTime<Utc>>,
  end_at: Option<DateTime<Utc>>,
}

pub async fn list(State(db): State<Database>) -> Result<impl IntoResponse, ResponseError> {
  let rows = tournament::Entity::find()
    .order_by_desc(tournament::Column::CreatedAt)
    .all(&db.conn)
    .await?;
  Ok(Json(rows))
}

pub async fn get_one(
  State(db): State<Database>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(access::tournament(&db, tournament_id).await?))
}

pub async fn create(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Json(input): Json<CreateTournament>,
) -> Result<impl IntoResponse, ResponseError> {
  access::authenticated(&token)?;
  if input.name.trim().is_empty() {
    return Err(ResponseError::BadRequest(
      "tournament name is required".to_owned(),
    ));
  }
  let min = input.team_size_min.unwrap_or(1);
  let max = input.team_size_max.unwrap_or(min);
  if min <= 0 || max < min {
    return Err(ResponseError::BadRequest(
      "invalid team size range".to_owned(),
    ));
  }
  let now = Utc::now();
  let txn = db.conn.begin().await?;
  let row = tournament::ActiveModel {
    id: Default::default(),
    name: Set(input.name.trim().to_owned()),
    brief: Set(input.brief),
    description: Set(input.description),
    owner_id: Set(token.id),
    lifecycle: Set(Lifecycle::Draft),
    competition_mode: Set(input.competition_mode.unwrap_or_default()),
    evidence_policy: Set(input.evidence_policy.unwrap_or_default()),
    leaderboard_visibility: Set(input.leaderboard_visibility.unwrap_or_default()),
    cover: Set(input.cover),
    team_size_min: Set(min),
    team_size_max: Set(max),
    registration_start_at: Set(input.registration_start_at),
    registration_end_at: Set(input.registration_end_at),
    start_at: Set(input.start_at),
    end_at: Set(input.end_at),
    created_at: Set(now),
    updated_at: Set(now),
  }
  .insert(&txn)
  .await?;
  tournament_staff::ActiveModel {
    id: Default::default(),
    tournament_id: Set(row.id),
    user_id: Set(token.id),
    role: Set(StaffRole::Owner),
    created_at: Set(now),
  }
  .insert(&txn)
  .await?;
  txn.commit().await?;
  Ok(Json(row))
}

#[derive(Deserialize)]
pub struct UpdateTournament {
  name: Option<String>,
  brief: Option<String>,
  description: Option<String>,
  lifecycle: Option<Lifecycle>,
  competition_mode: Option<CompetitionMode>,
  evidence_policy: Option<EvidencePolicy>,
  leaderboard_visibility: Option<LeaderboardVisibility>,
  cover: Option<String>,
  team_size_min: Option<i32>,
  team_size_max: Option<i32>,
  registration_start_at: Option<DateTime<Utc>>,
  registration_end_at: Option<DateTime<Utc>>,
  start_at: Option<DateTime<Utc>>,
  end_at: Option<DateTime<Utc>>,
}

pub async fn update(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<UpdateTournament>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let current = access::tournament(&db, tournament_id).await?;
  let recompute = input
    .lifecycle
    .is_some_and(|value| value != current.lifecycle)
    || input
      .leaderboard_visibility
      .is_some_and(|value| value != current.leaderboard_visibility);
  if let Some(next) = input.lifecycle
    && next != current.lifecycle
    && !current.lifecycle.can_transition_to(next)
  {
    return Err(ResponseError::Conflict(
      "invalid tournament lifecycle transition".to_owned(),
    ));
  }
  let min = input.team_size_min.unwrap_or(current.team_size_min);
  let max = input.team_size_max.unwrap_or(current.team_size_max);
  if min <= 0 || max < min {
    return Err(ResponseError::BadRequest(
      "invalid team size range".to_owned(),
    ));
  }
  let mut row = current.into_active_model();
  if let Some(value) = input.name {
    row.name = Set(value);
  }
  if let Some(value) = input.brief {
    row.brief = Set(value);
  }
  if input.description.is_some() {
    row.description = Set(input.description);
  }
  if let Some(value) = input.lifecycle {
    row.lifecycle = Set(value);
  }
  if let Some(value) = input.competition_mode {
    row.competition_mode = Set(value);
  }
  if let Some(value) = input.evidence_policy {
    row.evidence_policy = Set(value);
  }
  if let Some(value) = input.leaderboard_visibility {
    row.leaderboard_visibility = Set(value);
  }
  if input.cover.is_some() {
    row.cover = Set(input.cover);
  }
  row.team_size_min = Set(min);
  row.team_size_max = Set(max);
  if input.registration_start_at.is_some() {
    row.registration_start_at = Set(input.registration_start_at);
  }
  if input.registration_end_at.is_some() {
    row.registration_end_at = Set(input.registration_end_at);
  }
  if input.start_at.is_some() {
    row.start_at = Set(input.start_at);
  }
  if input.end_at.is_some() {
    row.end_at = Set(input.end_at);
  }
  row.updated_at = Set(Utc::now());
  let updated = row.update(&db.conn).await?;
  if recompute {
    scoring::recompute_now(&db, tournament_id).await?;
  }
  Ok(Json(updated))
}

pub async fn delete_one(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  let is_admin = token.permissions.0.iter().any(|p| {
    matches!(
      p,
      Permission::DevOps | Permission::Statistics | Permission::User
    )
  });
  if !is_admin {
    access::require_owner(&db, tournament_id, &token).await?;
  }
  let row = access::tournament(&db, tournament_id).await?;
  if row.lifecycle != Lifecycle::Draft && !is_admin {
    return Err(ResponseError::Conflict(
      "only draft tournaments can be deleted".to_owned(),
    ));
  }
  tournament::Entity::delete_by_id(tournament_id)
    .exec(&db.conn)
    .await?;
  Ok(())
}

pub async fn list_staff(
  State(db): State<Database>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::tournament(&db, tournament_id).await?;
  Ok(Json(
    tournament_staff::Entity::find()
      .filter(tournament_staff::Column::TournamentId.eq(tournament_id))
      .order_by_asc(tournament_staff::Column::Id)
      .all(&db.conn)
      .await?,
  ))
}

#[derive(Deserialize)]
pub struct StaffInput {
  user_id: i64,
  role: StaffRole,
}

pub async fn add_staff(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<StaffInput>,
) -> Result<impl IntoResponse, ResponseError> {
  let actor = access::require_organizer(&db, tournament_id, &token).await?;
  if input.role == StaffRole::Owner
    || (actor == StaffRole::Organizer && input.role != StaffRole::Judge)
  {
    return Err(ResponseError::Forbidden(
      "only the owner can manage organizers".to_owned(),
    ));
  }
  if tournament_staff::Entity::find()
    .filter(tournament_staff::Column::TournamentId.eq(tournament_id))
    .filter(tournament_staff::Column::UserId.eq(input.user_id))
    .one(&db.conn)
    .await?
    .is_some()
  {
    return Err(ResponseError::Conflict(
      "user is already tournament staff".to_owned(),
    ));
  }
  Ok(Json(
    tournament_staff::ActiveModel {
      id: Default::default(),
      tournament_id: Set(tournament_id),
      user_id: Set(input.user_id),
      role: Set(input.role),
      created_at: Set(Utc::now()),
    }
    .insert(&db.conn)
    .await?,
  ))
}

#[derive(Deserialize)]
pub struct StaffRoleInput {
  role: StaffRole,
}

pub async fn update_staff(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, user_id)): Path<(i64, i64)>, Json(input): Json<StaffRoleInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_owner(&db, tournament_id, &token).await?;
  if input.role == StaffRole::Owner {
    return Err(ResponseError::BadRequest(
      "ownership transfer is not supported".to_owned(),
    ));
  }
  let staff = tournament_staff::Entity::find()
    .filter(tournament_staff::Column::TournamentId.eq(tournament_id))
    .filter(tournament_staff::Column::UserId.eq(user_id))
    .one(&db.conn)
    .await?
    .ok_or_else(|| ResponseError::NotFound("staff member not found".to_owned()))?;
  if staff.role == StaffRole::Owner {
    return Err(ResponseError::Conflict(
      "the owner role cannot be changed".to_owned(),
    ));
  }
  let mut active = staff.into_active_model();
  active.role = Set(input.role);
  Ok(Json(active.update(&db.conn).await?))
}

pub async fn delete_staff(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, user_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, ResponseError> {
  let actor = access::require_organizer(&db, tournament_id, &token).await?;
  let staff = tournament_staff::Entity::find()
    .filter(tournament_staff::Column::TournamentId.eq(tournament_id))
    .filter(tournament_staff::Column::UserId.eq(user_id))
    .one(&db.conn)
    .await?
    .ok_or_else(|| ResponseError::NotFound("staff member not found".to_owned()))?;
  if staff.role == StaffRole::Owner
    || (staff.role == StaffRole::Organizer && actor != StaffRole::Owner)
  {
    return Err(ResponseError::Forbidden(
      "only the owner can remove organizers".to_owned(),
    ));
  }
  tournament_staff::Entity::delete_by_id(staff.id)
    .exec(&db.conn)
    .await?;
  Ok(())
}

#[derive(Deserialize)]
pub struct RoundInput {
  name: String,
  description: Option<String>,
  order_index: i32,
  start_at: Option<DateTime<Utc>>,
  end_at: Option<DateTime<Utc>>,
}

pub async fn list_rounds(
  State(db): State<Database>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::tournament(&db, tournament_id).await?;
  Ok(Json(
    tournament_round::Entity::find()
      .filter(tournament_round::Column::TournamentId.eq(tournament_id))
      .order_by_asc(tournament_round::Column::OrderIndex)
      .all(&db.conn)
      .await?,
  ))
}

pub async fn create_round(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<RoundInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  Ok(Json(
    tournament_round::ActiveModel {
      id: Default::default(),
      tournament_id: Set(tournament_id),
      name: Set(input.name),
      description: Set(input.description),
      order_index: Set(input.order_index),
      start_at: Set(input.start_at),
      end_at: Set(input.end_at),
    }
    .insert(&db.conn)
    .await?,
  ))
}

pub async fn update_round(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, round_id)): Path<(i64, i64)>, Json(input): Json<RoundInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let current = tournament_round::Entity::find_by_id(round_id)
    .one(&db.conn)
    .await?
    .filter(|row| row.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("round not found".to_owned()))?;
  let mut row = current.into_active_model();
  row.name = Set(input.name);
  row.description = Set(input.description);
  row.order_index = Set(input.order_index);
  row.start_at = Set(input.start_at);
  row.end_at = Set(input.end_at);
  Ok(Json(row.update(&db.conn).await?))
}

pub async fn delete_round(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, round_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let row = tournament_round::Entity::find_by_id(round_id)
    .one(&db.conn)
    .await?
    .filter(|row| row.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("round not found".to_owned()))?;
  tournament_round::Entity::delete_by_id(row.id)
    .exec(&db.conn)
    .await?;
  Ok(())
}

#[derive(Deserialize)]
pub struct ChartTagInput {
  round_id: i64,
  name: String,
  order_index: i32,
}

pub async fn list_chart_tags(
  State(db): State<Database>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::tournament(&db, tournament_id).await?;
  Ok(Json(
    chart_tag::Entity::find()
      .filter(chart_tag::Column::TournamentId.eq(tournament_id))
      .order_by_asc(chart_tag::Column::RoundId)
      .order_by_asc(chart_tag::Column::OrderIndex)
      .all(&db.conn)
      .await?,
  ))
}

pub async fn create_chart_tag(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<ChartTagInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  ensure_round(&db, tournament_id, input.round_id).await?;
  let name = input.name.trim();
  if name.is_empty() {
    return Err(ResponseError::BadRequest("tag name is required".to_owned()));
  }
  Ok(Json(
    chart_tag::ActiveModel {
      id: Default::default(),
      tournament_id: Set(tournament_id),
      round_id: Set(input.round_id),
      name: Set(name.to_owned()),
      order_index: Set(input.order_index),
    }
    .insert(&db.conn)
    .await?,
  ))
}

pub async fn update_chart_tag(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, tag_id)): Path<(i64, i64)>, Json(input): Json<ChartTagInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  ensure_round(&db, tournament_id, input.round_id).await?;
  let name = input.name.trim();
  if name.is_empty() {
    return Err(ResponseError::BadRequest("tag name is required".to_owned()));
  }
  let current = chart_tag::Entity::find_by_id(tag_id)
    .one(&db.conn)
    .await?
    .filter(|row| row.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("chart tag not found".to_owned()))?;
  if current.round_id != input.round_id
    && chart::Entity::find()
      .filter(chart::Column::TagId.eq(tag_id))
      .count(&db.conn)
      .await?
      > 0
  {
    return Err(ResponseError::Conflict(
      "chart tag with charts cannot move to another round".to_owned(),
    ));
  }
  let mut row = current.into_active_model();
  row.round_id = Set(input.round_id);
  row.name = Set(name.to_owned());
  row.order_index = Set(input.order_index);
  Ok(Json(row.update(&db.conn).await?))
}

pub async fn delete_chart_tag(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, tag_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let row = chart_tag::Entity::find_by_id(tag_id)
    .one(&db.conn)
    .await?
    .filter(|row| row.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("chart tag not found".to_owned()))?;
  let chart_count = chart::Entity::find()
    .filter(chart::Column::TagId.eq(tag_id))
    .count(&db.conn)
    .await?;
  if chart_count > 0 {
    return Err(ResponseError::Conflict(
      "chart tag is still used by charts".to_owned(),
    ));
  }
  chart_tag::Entity::delete_by_id(row.id)
    .exec(&db.conn)
    .await?;
  Ok(())
}

#[derive(Deserialize)]
pub struct ChartInput {
  round_id: i64,
  tag_id: i64,
  title: String,
  #[serde(default)]
  artist: String,
  #[serde(default)]
  charter: String,
  difficulty: String,
  #[serde(default)]
  level_constant: f64,
  cover: Option<String>,
  order_index: i32,
  weight_millionths: Option<i64>,
  #[serde(default = "empty_object")]
  metadata: Value,
}
fn empty_object() -> Value {
  json!({})
}

pub async fn list_charts(
  State(db): State<Database>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::tournament(&db, tournament_id).await?;
  Ok(Json(
    chart::Entity::find()
      .filter(chart::Column::TournamentId.eq(tournament_id))
      .order_by_asc(chart::Column::RoundId)
      .order_by_asc(chart::Column::TagId)
      .order_by_asc(chart::Column::OrderIndex)
      .all(&db.conn)
      .await?,
  ))
}

async fn ensure_round(
  db: &Database, tournament_id: i64, round_id: i64,
) -> Result<(), ResponseError> {
  if tournament_round::Entity::find_by_id(round_id)
    .one(&db.conn)
    .await?
    .is_some_and(|r| r.tournament_id == tournament_id)
  {
    Ok(())
  } else {
    Err(ResponseError::BadRequest(
      "round does not belong to tournament".to_owned(),
    ))
  }
}

async fn ensure_chart_tag(
  db: &Database, tournament_id: i64, round_id: i64, tag_id: i64,
) -> Result<(), ResponseError> {
  if chart_tag::Entity::find_by_id(tag_id)
    .one(&db.conn)
    .await?
    .is_some_and(|tag| tag.tournament_id == tournament_id && tag.round_id == round_id)
  {
    Ok(())
  } else {
    Err(ResponseError::BadRequest(
      "chart tag does not belong to tournament round".to_owned(),
    ))
  }
}

pub async fn create_chart(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<ChartInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  ensure_round(&db, tournament_id, input.round_id).await?;
  ensure_chart_tag(&db, tournament_id, input.round_id, input.tag_id).await?;
  Ok(Json(
    chart::ActiveModel {
      id: Default::default(),
      tournament_id: Set(tournament_id),
      round_id: Set(input.round_id),
      tag_id: Set(input.tag_id),
      title: Set(input.title),
      artist: Set(input.artist),
      charter: Set(input.charter),
      difficulty: Set(input.difficulty),
      level_constant: Set(input.level_constant),
      cover: Set(input.cover),
      order_index: Set(input.order_index),
      weight_millionths: Set(input.weight_millionths.unwrap_or(1_000_000)),
      metadata: Set(input.metadata),
    }
    .insert(&db.conn)
    .await?,
  ))
}

pub async fn update_chart(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, chart_id)): Path<(i64, i64)>, Json(input): Json<ChartInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  ensure_round(&db, tournament_id, input.round_id).await?;
  ensure_chart_tag(&db, tournament_id, input.round_id, input.tag_id).await?;
  let current = chart::Entity::find_by_id(chart_id)
    .one(&db.conn)
    .await?
    .filter(|r| r.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("chart not found".to_owned()))?;
  let mut row = current.into_active_model();
  row.round_id = Set(input.round_id);
  row.tag_id = Set(input.tag_id);
  row.title = Set(input.title);
  row.artist = Set(input.artist);
  row.charter = Set(input.charter);
  row.difficulty = Set(input.difficulty);
  row.level_constant = Set(input.level_constant);
  row.cover = Set(input.cover);
  row.order_index = Set(input.order_index);
  row.weight_millionths = Set(input.weight_millionths.unwrap_or(1_000_000));
  row.metadata = Set(input.metadata);
  Ok(Json(row.update(&db.conn).await?))
}

pub async fn delete_chart(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, chart_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let row = chart::Entity::find_by_id(chart_id)
    .one(&db.conn)
    .await?
    .filter(|r| r.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("chart not found".to_owned()))?;
  chart::Entity::delete_by_id(row.id).exec(&db.conn).await?;
  Ok(())
}

#[derive(Deserialize)]
pub struct RegistrationInput {
  display_name: Option<String>,
}

async fn registration_for_user(
  db: &Database, tournament_id: i64, user_id: i64,
) -> Result<Option<registration::Model>, ResponseError> {
  Ok(
    registration::Entity::find()
      .filter(registration::Column::TournamentId.eq(tournament_id))
      .filter(registration::Column::UserId.eq(user_id))
      .one(&db.conn)
      .await?,
  )
}

pub async fn list_registrations(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_judge(&db, tournament_id, &token).await?;
  Ok(Json(
    registration::Entity::find()
      .filter(registration::Column::TournamentId.eq(tournament_id))
      .order_by_asc(registration::Column::Id)
      .all(&db.conn)
      .await?,
  ))
}

pub async fn register(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<RegistrationInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::authenticated(&token)?;
  let tournament = access::tournament(&db, tournament_id).await?;
  if tournament.lifecycle != Lifecycle::Registration {
    return Err(ResponseError::Conflict(
      "tournament is not accepting registrations".to_owned(),
    ));
  }
  if registration_for_user(&db, tournament_id, token.id)
    .await?
    .is_some()
  {
    return Err(ResponseError::Conflict(
      "user is already registered".to_owned(),
    ));
  }
  let now = Utc::now();
  Ok(Json(
    registration::ActiveModel {
      id: Default::default(),
      tournament_id: Set(tournament_id),
      user_id: Set(token.id),
      display_name: Set(
        input
          .display_name
          .filter(|s| !s.trim().is_empty())
          .unwrap_or(token.nickname),
      ),
      status: Set(RegistrationStatus::Approved),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&db.conn)
    .await?,
  ))
}

pub async fn my_registration(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::authenticated(&token)?;
  Ok(Json(
    registration_for_user(&db, tournament_id, token.id).await?,
  ))
}

pub async fn withdraw(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::authenticated(&token)?;
  let tournament = access::tournament(&db, tournament_id).await?;
  if tournament.lifecycle.locks_teams() {
    return Err(ResponseError::Conflict("registration is locked".to_owned()));
  }
  let current = registration_for_user(&db, tournament_id, token.id)
    .await?
    .ok_or_else(|| ResponseError::NotFound("registration not found".to_owned()))?;
  let mut row = current.into_active_model();
  row.status = Set(RegistrationStatus::Withdrawn);
  row.updated_at = Set(Utc::now());
  Ok(Json(row.update(&db.conn).await?))
}

pub async fn list_teams(
  State(db): State<Database>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::tournament(&db, tournament_id).await?;
  Ok(Json(
    tournament_team::Entity::find()
      .filter(tournament_team::Column::TournamentId.eq(tournament_id))
      .order_by_asc(tournament_team::Column::Id)
      .all(&db.conn)
      .await?,
  ))
}

#[derive(Deserialize)]
pub struct TeamInput {
  name: String,
}

pub async fn create_team(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<TeamInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::authenticated(&token)?;
  let tournament = access::tournament(&db, tournament_id).await?;
  if tournament.lifecycle.locks_teams() {
    return Err(ResponseError::Conflict(
      "team membership is locked".to_owned(),
    ));
  }
  if tournament.competition_mode == CompetitionMode::Individual {
    return Err(ResponseError::Conflict(
      "tournament has no team competition".to_owned(),
    ));
  }
  let registration = registration_for_user(&db, tournament_id, token.id)
    .await?
    .filter(|r| r.status == RegistrationStatus::Approved)
    .ok_or_else(|| ResponseError::Forbidden("approved registration required".to_owned()))?;
  if team_member::Entity::find()
    .filter(team_member::Column::TournamentId.eq(tournament_id))
    .filter(team_member::Column::RegistrationId.eq(registration.id))
    .one(&db.conn)
    .await?
    .is_some()
  {
    return Err(ResponseError::Conflict(
      "registration already belongs to a team".to_owned(),
    ));
  }
  let txn = db.conn.begin().await?;
  let team = tournament_team::ActiveModel {
    id: Default::default(),
    tournament_id: Set(tournament_id),
    name: Set(input.name),
    captain_registration_id: Set(registration.id),
    created_at: Set(Utc::now()),
  }
  .insert(&txn)
  .await?;
  team_member::ActiveModel {
    id: Default::default(),
    tournament_id: Set(tournament_id),
    team_id: Set(team.id),
    registration_id: Set(registration.id),
    joined_at: Set(Utc::now()),
  }
  .insert(&txn)
  .await?;
  txn.commit().await?;
  Ok(Json(team))
}

pub async fn join_team(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, team_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, ResponseError> {
  access::authenticated(&token)?;
  let tournament = access::tournament(&db, tournament_id).await?;
  if tournament.lifecycle.locks_teams() {
    return Err(ResponseError::Conflict(
      "team membership is locked".to_owned(),
    ));
  }
  let team = tournament_team::Entity::find_by_id(team_id)
    .one(&db.conn)
    .await?
    .filter(|t| t.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("team not found".to_owned()))?;
  let registration = registration_for_user(&db, tournament_id, token.id)
    .await?
    .filter(|r| r.status == RegistrationStatus::Approved)
    .ok_or_else(|| ResponseError::Forbidden("approved registration required".to_owned()))?;
  if team_member::Entity::find()
    .filter(team_member::Column::TournamentId.eq(tournament_id))
    .filter(team_member::Column::RegistrationId.eq(registration.id))
    .one(&db.conn)
    .await?
    .is_some()
  {
    return Err(ResponseError::Conflict(
      "registration already belongs to a team".to_owned(),
    ));
  }
  let count = team_member::Entity::find()
    .filter(team_member::Column::TeamId.eq(team.id))
    .count(&db.conn)
    .await?;
  if count >= tournament.team_size_max as u64 {
    return Err(ResponseError::Conflict("team is full".to_owned()));
  }
  Ok(Json(
    team_member::ActiveModel {
      id: Default::default(),
      tournament_id: Set(tournament_id),
      team_id: Set(team.id),
      registration_id: Set(registration.id),
      joined_at: Set(Utc::now()),
    }
    .insert(&db.conn)
    .await?,
  ))
}

pub async fn leave_team(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::authenticated(&token)?;
  let tournament = access::tournament(&db, tournament_id).await?;
  if tournament.lifecycle.locks_teams() {
    return Err(ResponseError::Conflict(
      "team membership is locked".to_owned(),
    ));
  }
  let registration = registration_for_user(&db, tournament_id, token.id)
    .await?
    .ok_or_else(|| ResponseError::NotFound("registration not found".to_owned()))?;
  let membership = team_member::Entity::find()
    .filter(team_member::Column::TournamentId.eq(tournament_id))
    .filter(team_member::Column::RegistrationId.eq(registration.id))
    .one(&db.conn)
    .await?
    .ok_or_else(|| ResponseError::NotFound("team membership not found".to_owned()))?;
  let team = tournament_team::Entity::find_by_id(membership.team_id)
    .one(&db.conn)
    .await?
    .ok_or_else(|| ResponseError::NotFound("team not found".to_owned()))?;
  if team.captain_registration_id == registration.id {
    return Err(ResponseError::Conflict(
      "captain cannot leave the team".to_owned(),
    ));
  }
  team_member::Entity::delete_by_id(membership.id)
    .exec(&db.conn)
    .await?;
  Ok(())
}
