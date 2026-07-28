use axum::{
  Extension, Json,
  extract::{Path, State},
  response::IntoResponse,
};
use chrono::Utc;
use r2s_database::{chat, notification, registration};
use r2s_event::{
  Event, EventManager,
  events::{EventContainer, TournamentEvent, TournamentEventType},
};
use r2s_migrator::Database;
use sea_orm::{
  ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
  QuerySelect,
};
use serde::Deserialize;

use super::access;
use crate::{middleware::auth::Token, traits::ResponseError};

#[derive(Deserialize)]
pub struct NotificationInput {
  title: String,
  content: String,
}

pub async fn list_notifications(
  State(db): State<Database>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::tournament(&db, tournament_id).await?;
  Ok(Json(
    notification::Entity::find()
      .filter(notification::Column::TournamentId.eq(tournament_id))
      .order_by_desc(notification::Column::PublishedAt)
      .all(&db.conn)
      .await?,
  ))
}

pub async fn publish_notification(
  State(db): State<Database>, State(event): State<EventManager>,
  Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<NotificationInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let title = input.title.trim();
  let content = input.content.trim();
  if title.is_empty() || title.chars().count() > 127 || content.is_empty() {
    return Err(ResponseError::BadRequest(
      "notification title and content are required".to_owned(),
    ));
  }
  let row = notification::ActiveModel {
    id: Default::default(),
    tournament_id: Set(tournament_id),
    title: Set(title.to_owned()),
    content: Set(content.to_owned()),
    publisher_id: Set(token.id),
    published_at: Set(Utc::now()),
  }
  .insert(&db.conn)
  .await?;
  event
    .broadcast(EventContainer {
      tournament_id,
      event: Event::Tournament(TournamentEvent {
        event_type: TournamentEventType::NewNotification,
        actor_id: Some(token.id),
        message: Some(row.title.clone()),
      }),
    })
    .await;
  Ok(Json(row))
}

pub async fn delete_notification(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, notification_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let row = notification::Entity::find_by_id(notification_id)
    .one(&db.conn)
    .await?
    .filter(|row| row.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("notification not found".to_owned()))?;
  notification::Entity::delete_by_id(row.id)
    .exec(&db.conn)
    .await?;
  Ok(())
}

async fn can_chat(db: &Database, tournament_id: i64, token: &Token) -> Result<bool, ResponseError> {
  access::authenticated(token)?;
  if access::role(db, tournament_id, token).await?.is_some() {
    return Ok(true);
  }
  Ok(
    registration::Entity::find()
      .filter(registration::Column::TournamentId.eq(tournament_id))
      .filter(registration::Column::UserId.eq(token.id))
      .filter(registration::Column::Status.eq(registration::RegistrationStatus::Approved))
      .one(&db.conn)
      .await?
      .is_some(),
  )
}

pub async fn list_chat(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::tournament(&db, tournament_id).await?;
  if !can_chat(&db, tournament_id, &token).await? {
    return Err(ResponseError::Forbidden(
      "approved registration or staff access required".to_owned(),
    ));
  }
  Ok(Json(
    chat::Entity::find()
      .filter(chat::Column::TournamentId.eq(tournament_id))
      .order_by_desc(chat::Column::CreatedAt)
      .limit(200)
      .all(&db.conn)
      .await?,
  ))
}

#[derive(Deserialize)]
pub struct ChatInput {
  content: String,
}

pub async fn send_chat(
  State(db): State<Database>, State(event): State<EventManager>,
  Extension(token): Extension<Token>, Path(tournament_id): Path<i64>, Json(input): Json<ChatInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::tournament(&db, tournament_id).await?;
  if !can_chat(&db, tournament_id, &token).await? {
    return Err(ResponseError::Forbidden(
      "approved registration or staff access required".to_owned(),
    ));
  }
  let content = input.content.trim();
  if content.is_empty() || content.chars().count() > 4000 {
    return Err(ResponseError::BadRequest(
      "chat message must contain between 1 and 4000 characters".to_owned(),
    ));
  }
  let is_staff = access::role(&db, tournament_id, &token).await?.is_some();
  let row = chat::ActiveModel {
    id: Default::default(),
    tournament_id: Set(tournament_id),
    user_id: Set(token.id),
    content: Set(content.to_owned()),
    is_staff: Set(is_staff),
    created_at: Set(Utc::now()),
  }
  .insert(&db.conn)
  .await?;
  event
    .broadcast(EventContainer {
      tournament_id,
      event: Event::Tournament(TournamentEvent {
        event_type: TournamentEventType::ChatMessage,
        actor_id: Some(token.id),
        message: None,
      }),
    })
    .await;
  Ok(Json(row))
}
