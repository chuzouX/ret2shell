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
  source_type: Option<String>,
  external_id: Option<String>,
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
  chart_library_id: Option<i64>,
  round_id: i64,
  tag_id: i64,
  order_index: i32,
  weight_millionths: Option<i64>,
  description: Option<String>,
  title: Option<String>,
  artist: Option<String>,
  charter: Option<String>,
  difficulty: Option<String>,
  level_constant: Option<f64>,
  cover: Option<String>,
  metadata: Option<Value>,
  visibility: Option<tournament_chart_library::Visibility>,
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

async fn source_id(
  db: &Database, source_type: Option<&str>, source_id: Option<i64>,
) -> Result<i64, ResponseError> {
  if let Some(source_type) = source_type {
    let source_type = source_type.trim().to_ascii_lowercase();
    if source_type.is_empty() {
      return Err(ResponseError::BadRequest(
        "chart source is invalid".to_owned(),
      ));
    }
    let row = r2s_database::chart_source::Entity::find()
      .filter(r2s_database::chart_source::Column::SourceType.eq(source_type))
      .one(&db.conn)
      .await?
      .ok_or_else(|| ResponseError::BadRequest("chart source is invalid".to_owned()))?;
    return Ok(row.id);
  }
  if let Some(source_id) = source_id {
    return Ok(source_id);
  }
  let row = r2s_database::chart_source::Entity::find()
    .filter(r2s_database::chart_source::Column::SourceType.eq("personal"))
    .one(&db.conn)
    .await?
    .ok_or_else(|| ResponseError::NotFound("personal chart source not found".to_owned()))?;
  Ok(row.id)
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
        COALESCE(cs.name, cs.source_type, 'Unknown') AS source,
        COALESCE(cs.source_type, 'unknown') AS source_type,
        COALESCE(string_agg(DISTINCT t.name, ', ' ORDER BY t.name), '') AS tournaments
      FROM chart_library cl
      LEFT JOIN chart_source cs ON cs.id = cl.source_id
      LEFT JOIN tournament_chart_library tcl ON tcl.chart_library_id = cl.id
      LEFT JOIN tournament t ON t.id = tcl.tournament_id
        AND (tcl.visibility = 'public'
          OR (tcl.visibility = 'after_archive' AND t.lifecycle = 'archived'))
      WHERE (
        NOT EXISTS (
          SELECT 1 FROM tournament_chart_library hidden_link WHERE hidden_link.chart_library_id = cl.id
        )
      ) OR EXISTS (
        SELECT 1
        FROM tournament_chart_library visible_link
        JOIN tournament visible_t ON visible_t.id = visible_link.tournament_id
        WHERE visible_link.chart_library_id = cl.id
          AND (visible_link.visibility = 'public'
            OR (visible_link.visibility = 'after_archive' AND visible_t.lifecycle = 'archived'))
      )
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
  let source_id = source_id(&db, input.source_type.as_deref(), input.source_id).await?;
  let now = Utc::now();
  Ok(Json(
    chart_library::ActiveModel {
      id: Default::default(),
      source_id: Set(source_id),
      external_id: Set(input.external_id),
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
  State(db): State<Database>, Extension(config): Extension<r2s_database::config::Model>,
  Extension(token): Extension<Token>, Json(input): Json<PhiraImportInput>,
) -> Result<impl IntoResponse, ResponseError> {
  if token.id <= 0 {
    return Err(ResponseError::Unauthorized(
      "authentication required".to_owned(),
    ));
  }
  let source_id = phira_source_id(&db).await?;
  let chart = phira::get_chart(
    config.phira.as_ref().map(|config| config.base_url.as_str()),
    input.external_id,
  )
  .await?;
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
  if input.visibility.is_none() {
    return Err(ResponseError::BadRequest(
      "chart visibility is required".to_owned(),
    ));
  }
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
  if let Some(chart_library_id) = input.chart_library_id {
    if chart_library::Entity::find_by_id(chart_library_id)
      .one(&db.conn)
      .await?
      .is_none()
    {
      return Err(ResponseError::NotFound(
        "chart library entry not found".to_owned(),
      ));
    }
  } else if input
    .title
    .as_deref()
    .is_none_or(|value| value.trim().is_empty())
  {
    return Err(ResponseError::BadRequest(
      "chart library entry or chart title is required".to_owned(),
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
    let chart = match link.chart_library_id {
      Some(chart_library_id) => {
        let chart = chart_library::Entity::find_by_id(chart_library_id)
          .one(&db.conn)
          .await?
          .ok_or_else(|| ResponseError::NotFound("chart library entry not found".to_owned()))?;
        let source = r2s_database::chart_source::Entity::find_by_id(chart.source_id)
          .one(&db.conn)
          .await?
          .ok_or_else(|| ResponseError::NotFound("chart source not found".to_owned()))?;
        json!({
          "id": chart.id,
          "source_id": chart.source_id,
          "source": source.name,
          "source_type": source.source_type,
          "external_id": chart.external_id,
          "created_by": chart.created_by,
          "title": chart.title,
          "artist": chart.artist,
          "charter": chart.charter,
          "difficulty": chart.difficulty,
          "level_constant": chart.level_constant,
          "cover": chart.cover,
          "metadata": chart.metadata,
          "created_at": chart.created_at,
          "updated_at": chart.updated_at,
        })
      }
      None => json!({
        "id": link.id,
        "title": link.title,
        "artist": link.artist,
        "charter": link.charter,
        "difficulty": link.difficulty,
        "level_constant": link.level_constant,
        "cover": link.cover,
        "metadata": link.metadata,
        "description": link.description,
      }),
    };
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
  let mut chart_library_id = input.chart_library_id;
  if chart_library_id.is_none()
    && input
      .visibility
      .is_some_and(|visibility| visibility != tournament_chart_library::Visibility::Private)
  {
    let source_id = source_id(&db, Some("personal"), None).await?;
    let now = Utc::now();
    let chart = chart_library::ActiveModel {
      id: Default::default(),
      source_id: Set(source_id),
      external_id: Set(None),
      created_by: Set(token.id),
      title: Set(input.title.clone().unwrap_or_default()),
      artist: Set(input.artist.clone().unwrap_or_default()),
      charter: Set(input.charter.clone().unwrap_or_default()),
      difficulty: Set(input.difficulty.clone().unwrap_or_default()),
      level_constant: Set(input.level_constant.unwrap_or_default()),
      cover: Set(input.cover.clone()),
      metadata: Set(input.metadata.clone().unwrap_or_else(empty_object)),
      created_at: Set(now),
      updated_at: Set(now),
    }
    .insert(&db.conn)
    .await?;
    chart_library_id = Some(chart.id);
  }
  Ok(Json(
    tournament_chart_library::ActiveModel {
      id: Default::default(),
      tournament_id: Set(tournament_id),
      chart_library_id: Set(chart_library_id),
      visibility: Set(
        input
          .visibility
          .ok_or_else(|| ResponseError::BadRequest("chart visibility is required".to_owned()))?,
      ),
      round_id: Set(input.round_id),
      tag_id: Set(input.tag_id),
      order_index: Set(input.order_index),
      weight_millionths: Set(input.weight_millionths.unwrap_or(1_000_000)),
      description: Set(input.description),
      title: Set(input.title.unwrap_or_default()),
      artist: Set(input.artist.unwrap_or_default()),
      charter: Set(input.charter.unwrap_or_default()),
      difficulty: Set(input.difficulty.unwrap_or_default()),
      level_constant: Set(input.level_constant.unwrap_or_default()),
      cover: Set(input.cover),
      metadata: Set(input.metadata.unwrap_or_else(empty_object)),
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
  row.visibility = Set(
    input
      .visibility
      .ok_or_else(|| ResponseError::BadRequest("chart visibility is required".to_owned()))?,
  );
  row.round_id = Set(input.round_id);
  row.tag_id = Set(input.tag_id);
  row.order_index = Set(input.order_index);
  row.weight_millionths = Set(input.weight_millionths.unwrap_or(1_000_000));
  row.description = Set(input.description);
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
