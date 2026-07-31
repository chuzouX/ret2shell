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
  tournament::{
    self, CompetitionMode, EvidencePolicy, LeaderboardVisibility, Lifecycle, LifecycleScheduleMode,
  },
  tournament_round,
  tournament_staff::{self, StaffRole},
  tournament_team,
  user::Permission,
};
use r2s_migrator::Database;
use sea_orm::{
  ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
  QueryFilter, QueryOrder, TransactionTrait, TryIntoModel,
};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{
  access,
  round_visibility::{self, Audience},
  scoring,
};
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
  rules: Option<String>,
  rules_visible: Option<bool>,
  announcements_visible: Option<bool>,
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
    rules: Set(input.rules),
    rules_visible: Set(input.rules_visible.unwrap_or(false)),
    announcements_visible: Set(input.announcements_visible.unwrap_or(false)),
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
    registration_schedule: Set(LifecycleScheduleMode::Manual),
    registration_at: Set(None),
    running_schedule: Set(LifecycleScheduleMode::Manual),
    running_at: Set(None),
    review_schedule: Set(LifecycleScheduleMode::Manual),
    review_at: Set(None),
    finished_schedule: Set(LifecycleScheduleMode::Manual),
    finished_at: Set(None),
    organizer_can_edit_archived: Set(false),
    current_round_id: Set(None),
    round_control_mode: Set(tournament::RoundControlMode::ManualAssisted),
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
  rules: Option<String>,
  rules_visible: Option<bool>,
  announcements_visible: Option<bool>,
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
  registration_schedule: Option<LifecycleScheduleMode>,
  registration_at: Option<Option<DateTime<Utc>>>,
  running_schedule: Option<LifecycleScheduleMode>,
  running_at: Option<Option<DateTime<Utc>>>,
  review_schedule: Option<LifecycleScheduleMode>,
  review_at: Option<Option<DateTime<Utc>>>,
  finished_schedule: Option<LifecycleScheduleMode>,
  finished_at: Option<Option<DateTime<Utc>>>,
  organizer_can_edit_archived: Option<bool>,
}

fn validate_lifecycle_schedule(
  created_at: DateTime<Utc>, stages: [(LifecycleScheduleMode, Option<DateTime<Utc>>, &str); 4],
) -> Result<(), ResponseError> {
  let mut previous = None;
  for (mode, at, name) in stages {
    if mode == LifecycleScheduleMode::Scheduled && at.is_none() {
      return Err(ResponseError::BadRequest(format!(
        "{name} stage start time is required for scheduled stages"
      )));
    }
    if let Some(value) = at {
      if value < created_at {
        return Err(ResponseError::BadRequest(format!(
          "{name} stage start time cannot be earlier than tournament creation"
        )));
      }
      if previous.is_some_and(|previous| value < previous) {
        return Err(ResponseError::BadRequest(
          "lifecycle stage start times must be in chronological order".to_owned(),
        ));
      }
      previous = Some(value);
    }
  }
  Ok(())
}

pub async fn update(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<UpdateTournament>,
) -> Result<impl IntoResponse, ResponseError> {
  let current = access::tournament(&db, tournament_id).await?;
  let is_devops = token.permissions.0.contains(&Permission::DevOps);
  let staff_role = access::role(&db, tournament_id, &token).await?;
  let can_edit = matches!(staff_role, Some(StaffRole::Owner | StaffRole::Organizer)) || is_devops;
  if !can_edit {
    return Err(ResponseError::Forbidden(
      "organizer access required".to_owned(),
    ));
  }
  if current.lifecycle == Lifecycle::Archived && !is_devops && !current.organizer_can_edit_archived
  {
    return Err(ResponseError::Forbidden(
      "archived tournament is locked".to_owned(),
    ));
  }
  if input.organizer_can_edit_archived.is_some() && !is_devops {
    return Err(ResponseError::Forbidden(
      "devops access required".to_owned(),
    ));
  }
  if current.lifecycle == Lifecycle::Archived
    && input
      .lifecycle
      .is_some_and(|value| value != Lifecycle::Archived)
    && !is_devops
  {
    return Err(ResponseError::Forbidden(
      "only devops can reopen archived tournaments".to_owned(),
    ));
  }
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
  let registration_schedule = input
    .registration_schedule
    .unwrap_or(current.registration_schedule);
  let registration_at = input.registration_at.unwrap_or(current.registration_at);
  let running_schedule = input.running_schedule.unwrap_or(current.running_schedule);
  let running_at = input.running_at.unwrap_or(current.running_at);
  let review_schedule = input.review_schedule.unwrap_or(current.review_schedule);
  let review_at = input.review_at.unwrap_or(current.review_at);
  let finished_schedule = input.finished_schedule.unwrap_or(current.finished_schedule);
  let finished_at = input.finished_at.unwrap_or(current.finished_at);
  validate_lifecycle_schedule(
    current.created_at,
    [
      (registration_schedule, registration_at, "registration"),
      (running_schedule, running_at, "running"),
      (review_schedule, review_at, "review"),
      (finished_schedule, finished_at, "finished"),
    ],
  )?;
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
  if input.rules.is_some() {
    row.rules = Set(input.rules);
  }
  if let Some(value) = input.rules_visible {
    row.rules_visible = Set(value);
  }
  if let Some(value) = input.announcements_visible {
    row.announcements_visible = Set(value);
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
  if input.registration_schedule.is_some() {
    row.registration_schedule = Set(registration_schedule);
  }
  if input.registration_at.is_some() {
    row.registration_at = Set(registration_at);
  }
  if input.running_schedule.is_some() {
    row.running_schedule = Set(running_schedule);
  }
  if input.running_at.is_some() {
    row.running_at = Set(running_at);
  }
  if input.review_schedule.is_some() {
    row.review_schedule = Set(review_schedule);
  }
  if input.review_at.is_some() {
    row.review_at = Set(review_at);
  }
  if input.finished_schedule.is_some() {
    row.finished_schedule = Set(finished_schedule);
  }
  if input.finished_at.is_some() {
    row.finished_at = Set(finished_at);
  }
  if let Some(value) = input.organizer_can_edit_archived {
    row.organizer_can_edit_archived = Set(value);
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
  #[serde(default = "default_release_audience")]
  release_audience: serde_json::Value,
  #[serde(default)]
  release_timing: tournament_round::ReleaseTiming,
  #[serde(default)]
  end_mode: tournament_round::EndMode,
  release_at: Option<DateTime<Utc>>,
}

fn default_release_audience() -> serde_json::Value {
  serde_json::json!(["staff"])
}

pub async fn list_rounds(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  let tournament = access::tournament(&db, tournament_id).await?;
  let audience = round_visibility::audience(&db, tournament_id, &token).await?;
  let rows = tournament_round::Entity::find()
    .filter(tournament_round::Column::TournamentId.eq(tournament_id))
    .order_by_asc(tournament_round::Column::OrderIndex)
    .all(&db.conn)
    .await?;
  Ok(Json(
    rows
      .into_iter()
      .filter(|row| {
        audience == Audience::Staff
          || round_visibility::is_visible(row, audience, tournament.current_round_id)
      })
      .collect::<Vec<_>>(),
  ))
}

pub async fn create_round(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<RoundInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  round_visibility::validate_audience(&input.release_audience)?;
  round_visibility::validate_schedule_values(
    input.start_at,
    input.end_at,
    input.release_at,
    input.end_mode,
  )?;
  Ok(Json(
    tournament_round::ActiveModel {
      id: Default::default(),
      tournament_id: Set(tournament_id),
      name: Set(input.name),
      description: Set(input.description),
      order_index: Set(input.order_index),
      start_at: Set(input.start_at),
      end_at: Set(input.end_at),
      release_audience: Set(input.release_audience),
      release_timing: Set(input.release_timing),
      end_mode: Set(input.end_mode),
      release_at: Set(input.release_at),
      started_at: Set(None),
      ended_at: Set(None),
      released_at: Set(None),
      manually_released: Set(false),
      manually_ended: Set(false),
      release_version: Set(0),
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
  round_visibility::validate_audience(&input.release_audience)?;
  row.release_audience = Set(input.release_audience);
  row.release_timing = Set(input.release_timing);
  row.end_mode = Set(input.end_mode);
  row.release_at = Set(input.release_at);
  let preview = row
    .clone()
    .try_into_model()
    .map_err(|_| ResponseError::BadRequest("invalid round release configuration".to_owned()))?;
  round_visibility::validate_schedule(&preview)?;
  Ok(Json(row.update(&db.conn).await?))
}

#[derive(Deserialize, Default)]
pub struct RoundActionInput {
  #[serde(default)]
  force: bool,
}

async fn round_for_tournament(
  db: &Database, tournament_id: i64, round_id: i64,
) -> Result<tournament_round::Model, ResponseError> {
  tournament_round::Entity::find_by_id(round_id)
    .one(&db.conn)
    .await?
    .filter(|row| row.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("round not found".to_owned()))
}

pub async fn enter_round(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, round_id)): Path<(i64, i64)>, Json(input): Json<RoundActionInput>,
) -> Result<impl IntoResponse, ResponseError> {
  let tournament = access::tournament(&db, tournament_id).await?;
  access::require_organizer(&db, tournament_id, &token).await?;
  if tournament.lifecycle != Lifecycle::Running {
    return Err(ResponseError::Conflict(
      "rounds can only be entered while tournament is running".to_owned(),
    ));
  }
  let round = round_for_tournament(&db, tournament_id, round_id).await?;
  if round.ended_at.is_some() {
    return Err(ResponseError::Conflict(
      "ended round cannot be entered".to_owned(),
    ));
  }
  if !input.force && round.start_at.is_some_and(|at| at > Utc::now()) {
    return Err(ResponseError::Conflict(format!(
      "round is scheduled for {}",
      round.start_at.unwrap()
    )));
  }
  let now = Utc::now();
  let current = tournament.current_round_id;
  let txn = db.conn.begin().await?;
  if let Some(current_id) = current.filter(|id| *id != round_id) {
    tournament_round::Entity::update_many()
      .col_expr(tournament_round::Column::EndedAt, now.into())
      .col_expr(tournament_round::Column::ReleaseVersion, (1i64).into())
      .filter(tournament_round::Column::Id.eq(current_id))
      .filter(tournament_round::Column::EndedAt.is_null())
      .exec(&txn)
      .await?;
  }
  tournament_round::Entity::update_many()
    .col_expr(tournament_round::Column::StartedAt, now.into())
    .col_expr(tournament_round::Column::ReleaseVersion, (1i64).into())
    .filter(tournament_round::Column::Id.eq(round_id))
    .filter(tournament_round::Column::EndedAt.is_null())
    .exec(&txn)
    .await?;
  let mut tournament_row = tournament.into_active_model();
  tournament_row.current_round_id = Set(Some(round_id));
  tournament_row.updated_at = Set(now);
  tournament_row.update(&txn).await?;
  txn.commit().await?;
  Ok(Json(
    round_for_tournament(&db, tournament_id, round_id).await?,
  ))
}

pub async fn release_round(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, round_id)): Path<(i64, i64)>, Json(_input): Json<RoundActionInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let round = round_for_tournament(&db, tournament_id, round_id).await?;
  let mut row = round.into_active_model();
  row.manually_released = Set(true);
  row.released_at = Set(Some(Utc::now()));
  row.release_version = Set(row.release_version.clone().unwrap() + 1);
  Ok(Json(row.update(&db.conn).await?))
}

pub async fn withdraw_round_release(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, round_id)): Path<(i64, i64)>, Json(_input): Json<RoundActionInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let round = round_for_tournament(&db, tournament_id, round_id).await?;
  if round.started_at.is_some() || round.ended_at.is_some() || !round.manually_released {
    return Err(ResponseError::Conflict(
      "round release cannot be withdrawn after entering the round".to_owned(),
    ));
  }
  let mut row = round.into_active_model();
  row.manually_released = Set(false);
  row.released_at = Set(None);
  row.release_version = Set(row.release_version.clone().unwrap() + 1);
  Ok(Json(row.update(&db.conn).await?))
}

pub async fn end_round(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, round_id)): Path<(i64, i64)>, Json(_input): Json<RoundActionInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let tournament = access::tournament(&db, tournament_id).await?;
  if tournament.lifecycle != Lifecycle::Running {
    return Err(ResponseError::Conflict(
      "rounds can only be ended while tournament is running".to_owned(),
    ));
  }
  let round = round_for_tournament(&db, tournament_id, round_id).await?;
  if tournament.current_round_id != Some(round_id)
    || round.started_at.is_none()
    || round.ended_at.is_some()
  {
    return Err(ResponseError::Conflict(
      "only the current active round can be ended".to_owned(),
    ));
  }
  let mut row = round.into_active_model();
  row.ended_at = Set(Some(Utc::now()));
  row.manually_ended = Set(true);
  row.release_version = Set(row.release_version.clone().unwrap() + 1);
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
