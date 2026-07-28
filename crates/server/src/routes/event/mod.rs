use std::net::IpAddr;

use axum::{
  Extension, Router,
  extract::{Query, State, WebSocketUpgrade},
  response::IntoResponse,
  routing::get,
};
use r2s_database::{registration, tournament, tournament_staff};
use r2s_event::EventManager;
use r2s_migrator::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use tracing::{info, warn};

use crate::{
  middleware::auth::Token,
  traits::{GlobalState, ResponseError},
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new().route("/connect", get(connect_tournament))
}

#[derive(Deserialize)]
struct ConnectQuery {
  pub tournament_id: i64,
  pub client: Option<String>,
}

async fn connect_tournament(
  State(ref db): State<Database>, State(event): State<EventManager>,
  Extension(ip): Extension<IpAddr>, Extension(token): Extension<Token>,
  Query(ConnectQuery {
    tournament_id,
    client,
  }): Query<ConnectQuery>, ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ResponseError> {
  if token.id <= 0 {
    return Err(ResponseError::Unauthorized("please login first".to_owned()));
  }
  let tournament_exists = tournament::Entity::find_by_id(tournament_id)
    .one(&db.conn)
    .await?
    .is_some();
  let registered = registration::Entity::find()
    .filter(registration::Column::TournamentId.eq(tournament_id))
    .filter(registration::Column::UserId.eq(token.id))
    .one(&db.conn)
    .await?
    .is_some();
  let staff = tournament_staff::Entity::find()
    .filter(tournament_staff::Column::TournamentId.eq(tournament_id))
    .filter(tournament_staff::Column::UserId.eq(token.id))
    .one(&db.conn)
    .await?
    .is_some();
  if tournament_exists && (registered || staff) {
    info!(
      client=%client.as_deref().unwrap_or("Unspecified v0.0.0"),
      %tournament_id,
      %ip,
      "tournament event connection established",
    );
    return Ok(ws.on_upgrade(move |ws| async move {
      event
        .subscribe(
          tournament_id,
          ip,
          client.unwrap_or("Unspecified v0.0.0".to_owned()),
          ws,
        )
        .await;
    }));
  }
  warn!(
    %tournament_id,
    %ip,
    "tournament event connection denied",
  );
  Err(ResponseError::Forbidden("permission denied".to_owned()))
}
