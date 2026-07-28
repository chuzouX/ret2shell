use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TournamentEventType {
  LifecycleChanged,
  LeaderboardUpdated,
  NewNotification,
  ResultReviewed,
  ChatMessage,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TournamentEvent {
  pub event_type: TournamentEventType,
  pub actor_id: Option<i64>,
  pub message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DevopsEventType {
  ServerPanic,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DevopsEvent {
  pub event_type: DevopsEventType,
  pub message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
  Tournament(TournamentEvent),
  Devops(DevopsEvent),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventContainer {
  pub tournament_id: i64,
  pub event: Event,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Broadcast {
  Publish(Box<EventContainer>),
  Heartbeat,
}
