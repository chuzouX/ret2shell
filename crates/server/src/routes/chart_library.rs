use axum::{
  Extension, Json,
  extract::{Path, State},
  response::IntoResponse,
};
use chrono::Utc;
use r2s_database::{chart_library, chart_tag, tournament_chart_library};
use r2s_migrator::Database;
use r2s_oauth::phira;
use sea_orm::{
  ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait,
  FromQueryResult, IntoActiveModel, QueryFilter, QueryOrder, Statement, Value as SeaValue,
};
use serde::Deserialize;
use serde_json::{Value, json};

use super::tournament::access;
use crate::{middleware::auth::Token, traits::ResponseError};

#[derive(Deserialize)]
pub struct ChartLibraryInput {
  source_id: Option<i64>,
  external_id: Option<String>,
  exclusive_tournament_id: Option<i64>,
  title: String,
  #[serde(default)]
  artist: String,
  #[serde(default)]
  charter: String,
  difficulty: String,
  #[serde(default)]
  level_constant: f64,
  cover: Option<String>,
  #[serde(default = "empty_object")]
  metadata: Value,
}

#[derive(Deserialize)]
pub struct ChartLibraryPatch {
  title: Option<String>,
  artist: Option<String>,
  charter: Option<String>,
  difficulty: Option<String>,
  level_constant: Option<f64>,
  cover: Option<String>,
  metadata: Option<Value>,
}

#[derive(Deserialize)]
pub struct PhiraImportInput {
  external_id: i64,
}

#[derive(Deserialize)]
pub struct LinkInput {
  chart_library_id: i64,
  round_id: i64,
  tag_id: i64,
  order_index: i32,
  weight_millionths: Option<i64>,
}

#[derive(serde::Serialize, FromQueryResult)]
pub struct ChartLibraryListItem {
  #[sea_orm(nested)]
  pub chart: chart_library::Model,
  pub source: String,
  pub source_type: String,
  pub tournaments: String,
}

fn empty_object() -> Value {
  json!({})
}

async fn phira_source_id(db: &Database) -> Result<i64, ResponseError> {
  let row = db
    .conn
    .query_one(Statement::from_sql_and_values(
      DatabaseBackend::Postgres,
      "SELECT id FROM chart_source WHERE source_type = $1 LIMIT 1",
      [SeaValue::from("phira")],
    ))
    .await?
    .ok_or_else(|| ResponseError::NotFound("phira chart source not found".to_owned()))?;
  Ok(row.try_get_by_index(0)?)
}

fn imported_chart(
  chart: phira::Chart, source_id: i64, created_by: i64,
) -> chart_library::ActiveModel {
  let metadata = chart.extra;
  let now = Utc::now();
  chart_library::ActiveModel {
    id: Default::default(),
    source_id: Set(source_id),
    external_id: Set(Some(chart.id.to_string())),
    exclusive_tournament_id: Set(None),
    created_by: Set(created_by),
    title: Set(chart.name),
    artist: Set(chart.composer),
    charter: Set(chart.charter),
    difficulty: Set(chart.level),
    level_constant: Set(chart.difficulty),
    cover: Set(chart.illustration),
    metadata: Set(serde_json::Value::Object(metadata)),
    created_at: Set(now),
    updated_at: Set(now),
  }
}

pub async fn list(State(db): State<Database>) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(
    ChartLibraryListItem::find_by_statement(Statement::from_string(
      DatabaseBackend::Postgres,
      r#"
      SELECT
        cl.*,
        CASE
          WHEN cl.exclusive_tournament_id IS NOT NULL THEN '赛事专属'
          ELSE COALESCE(cs.name, cs.source_type, 'Unknown')
        END AS source,
        CASE
          WHEN cl.exclusive_tournament_id IS NOT NULL THEN 'exclusive'
          ELSE COALESCE(cs.source_type, 'unknown')
        END AS source_type,
        COALESCE(string_agg(DISTINCT t.name, ', ' ORDER BY t.name), '') AS tournaments
      FROM chart_library cl
      LEFT JOIN chart_source cs ON cs.id = cl.source_id
      LEFT JOIN tournament_chart_library tcl ON tcl.chart_library_id = cl.id
      LEFT JOIN tournament t ON t.id = tcl.tournament_id AND t.lifecycle = 'archived'
      GROUP BY cl.id, cs.name, cs.source_type
      ORDER BY cl.id ASC
      "#,
    ))
    .all(&db.conn)
    .await?,
  ))
}

pub async fn create(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Json(input): Json<ChartLibraryInput>,
) -> Result<impl IntoResponse, ResponseError> {
  if token.id <= 0 {
    return Err(ResponseError::Unauthorized(
      "authentication required".to_owned(),
    ));
  }
  let source_id = input
    .source_id
    .ok_or_else(|| ResponseError::BadRequest("chart source is required".to_owned()))?;
  let now = Utc::now();
  Ok(Json(
    chart_library::ActiveModel {
      id: Default::default(),
      source_id: Set(source_id),
      external_id: Set(input.external_id),
      exclusive_tournament_id: Set(input.exclusive_tournament_id),
      created_by: Set(token.id),
      title: Set(input.title),
      artist: Set(input.artist),
      charter: Set(input.charter),
      difficulty: Set(input.difficulty),
      level_constant: Set(input.level_constant),
      cover: Set(input.cover),
      metadata: Set(input.metadata),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&db.conn)
    .await?,
  ))
}

pub async fn import_phira(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Json(input): Json<PhiraImportInput>,
) -> Result<impl IntoResponse, ResponseError> {
  if token.id <= 0 {
    return Err(ResponseError::Unauthorized(
      "authentication required".to_owned(),
    ));
  }
  let source_id = phira_source_id(&db).await?;
  let chart = phira::get_chart(input.external_id).await?;
  let external_id = chart.id.to_string();
  if let Some(existing) = chart_library::Entity::find()
    .filter(chart_library::Column::SourceId.eq(source_id))
    .filter(chart_library::Column::ExternalId.eq(external_id))
    .one(&db.conn)
    .await?
  {
    return Ok(Json(existing));
  }
  Ok(Json(
    imported_chart(chart, source_id, token.id)
      .insert(&db.conn)
      .await?,
  ))
}

pub async fn update(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(id): Path<i64>,
  Json(input): Json<ChartLibraryPatch>,
) -> Result<impl IntoResponse, ResponseError> {
  if token.id <= 0 {
    return Err(ResponseError::Unauthorized(
      "authentication required".to_owned(),
    ));
  }
  let current = chart_library::Entity::find_by_id(id)
    .one(&db.conn)
    .await?
    .ok_or_else(|| ResponseError::NotFound("chart library entry not found".to_owned()))?;
  let mut row = current.into_active_model();
  if let Some(value) = input.title {
    row.title = Set(value);
  }
  if let Some(value) = input.artist {
    row.artist = Set(value);
  }
  if let Some(value) = input.charter {
    row.charter = Set(value);
  }
  if let Some(value) = input.difficulty {
    row.difficulty = Set(value);
  }
  if let Some(value) = input.level_constant {
    row.level_constant = Set(value);
  }
  if input.cover.is_some() {
    row.cover = Set(input.cover);
  }
  if let Some(value) = input.metadata {
    row.metadata = Set(value);
  }
  Ok(Json(row.update(&db.conn).await?))
}

pub async fn delete(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  if token.id <= 0 {
    return Err(ResponseError::Unauthorized(
      "authentication required".to_owned(),
    ));
  }
  chart_library::Entity::delete_by_id(id)
    .exec(&db.conn)
    .await?;
  Ok(())
}

async fn ensure_link_target(
  db: &Database, tournament_id: i64, input: &LinkInput,
) -> Result<(), ResponseError> {
  let round = r2s_database::tournament_round::Entity::find_by_id(input.round_id)
    .one(&db.conn)
    .await?;
  let tag = chart_tag::Entity::find_by_id(input.tag_id)
    .one(&db.conn)
    .await?;
  if round.is_none_or(|row| row.tournament_id != tournament_id)
    || tag.is_none_or(|row| row.tournament_id != tournament_id || row.round_id != input.round_id)
  {
    return Err(ResponseError::BadRequest(
      "round or chart tag does not belong to tournament".to_owned(),
    ));
  }
  if chart_library::Entity::find_by_id(input.chart_library_id)
    .one(&db.conn)
    .await?
    .is_none()
  {
    return Err(ResponseError::NotFound(
      "chart library entry not found".to_owned(),
    ));
  }
  Ok(())
}

pub async fn list_links(
  State(db): State<Database>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::tournament(&db, tournament_id).await?;
  let links = tournament_chart_library::Entity::find()
    .filter(tournament_chart_library::Column::TournamentId.eq(tournament_id))
    .order_by_asc(tournament_chart_library::Column::RoundId)
    .order_by_asc(tournament_chart_library::Column::OrderIndex)
    .all(&db.conn)
    .await?;
  let mut response = Vec::with_capacity(links.len());
  for link in links {
    let chart = chart_library::Entity::find_by_id(link.chart_library_id)
      .one(&db.conn)
      .await?
      .ok_or_else(|| ResponseError::NotFound("chart library entry not found".to_owned()))?;
    response.push(json!({"link": link, "chart": chart}));
  }
  Ok(Json(response))
}

pub async fn create_link(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<LinkInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  ensure_link_target(&db, tournament_id, &input).await?;
  Ok(Json(
    tournament_chart_library::ActiveModel {
      id: Default::default(),
      tournament_id: Set(tournament_id),
      chart_library_id: Set(input.chart_library_id),
      round_id: Set(input.round_id),
      tag_id: Set(input.tag_id),
      order_index: Set(input.order_index),
      weight_millionths: Set(input.weight_millionths.unwrap_or(1_000_000)),
    }
    .insert(&db.conn)
    .await?,
  ))
}

pub async fn update_link(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, id)): Path<(i64, i64)>, Json(input): Json<LinkInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  ensure_link_target(&db, tournament_id, &input).await?;
  let current = tournament_chart_library::Entity::find_by_id(id)
    .one(&db.conn)
    .await?
    .filter(|row| row.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("tournament chart link not found".to_owned()))?;
  let mut row = current.into_active_model();
  row.chart_library_id = Set(input.chart_library_id);
  row.round_id = Set(input.round_id);
  row.tag_id = Set(input.tag_id);
  row.order_index = Set(input.order_index);
  row.weight_millionths = Set(input.weight_millionths.unwrap_or(1_000_000));
  Ok(Json(row.update(&db.conn).await?))
}

pub async fn delete_link(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let row = tournament_chart_library::Entity::find_by_id(id)
    .one(&db.conn)
    .await?
    .filter(|row| row.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("tournament chart link not found".to_owned()))?;
  tournament_chart_library::Entity::delete_by_id(row.id)
    .exec(&db.conn)
    .await?;
  Ok(())
}
