use axum::{
  Router,
  routing::{delete, get, patch, post},
};

use crate::traits::GlobalState;

pub(crate) mod access;
mod communication;
mod core;
mod results;
pub(crate) mod round_visibility;
pub(crate) mod scoring;

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .route("/", get(core::list).post(core::create))
    .route(
      "/{tournament}",
      get(core::get_one)
        .patch(core::update)
        .delete(core::delete_one),
    )
    .route(
      "/{tournament}/staff",
      get(core::list_staff).post(core::add_staff),
    )
    .route(
      "/{tournament}/staff/{user}",
      patch(core::update_staff).delete(core::delete_staff),
    )
    .route(
      "/{tournament}/rounds",
      get(core::list_rounds).post(core::create_round),
    )
    .route(
      "/{tournament}/rounds/{round}",
      patch(core::update_round).delete(core::delete_round),
    )
    .route(
      "/{tournament}/rounds/{round}/enter",
      post(core::enter_round),
    )
    .route(
      "/{tournament}/rounds/{round}/release",
      post(core::release_round),
    )
    .route(
      "/{tournament}/rounds/{round}/withdraw-release",
      post(core::withdraw_round_release),
    )
    .route("/{tournament}/rounds/{round}/end", post(core::end_round))
    .route(
      "/{tournament}/charts",
      get(core::list_charts).post(core::create_chart),
    )
    .route(
      "/{tournament}/chart-library",
      get(super::chart_library::list_links).post(super::chart_library::create_link),
    )
    .route(
      "/{tournament}/chart-library/{link}",
      patch(super::chart_library::update_link).delete(super::chart_library::delete_link),
    )
    .route(
      "/{tournament}/chart-tags",
      get(core::list_chart_tags).post(core::create_chart_tag),
    )
    .route(
      "/{tournament}/chart-tags/{tag}",
      patch(core::update_chart_tag).delete(core::delete_chart_tag),
    )
    .route(
      "/{tournament}/charts/{chart}",
      patch(core::update_chart).delete(core::delete_chart),
    )
    .route(
      "/{tournament}/registrations",
      get(core::list_registrations).post(core::register),
    )
    .route(
      "/{tournament}/registrations/me",
      get(core::my_registration).delete(core::withdraw),
    )
    .route(
      "/{tournament}/teams",
      get(core::list_teams).post(core::create_team),
    )
    .route("/{tournament}/teams/{team}/join", post(core::join_team))
    .route("/{tournament}/teams/leave", delete(core::leave_team))
    .route(
      "/{tournament}/notifications",
      get(communication::list_notifications).post(communication::publish_notification),
    )
    .route(
      "/{tournament}/notifications/{notification}",
      delete(communication::delete_notification),
    )
    .route(
      "/{tournament}/chat",
      get(communication::list_chat).post(communication::send_chat),
    )
    .route(
      "/{tournament}/results",
      get(results::list).post(results::submit),
    )
    .route(
      "/{tournament}/results/import/preview",
      post(results::preview_import),
    )
    .route("/{tournament}/results/import", post(results::commit_import))
    .route(
      "/{tournament}/results/{result}/review",
      post(results::review),
    )
    .route(
      "/{tournament}/results/{result}/correct",
      post(results::correct),
    )
    .route(
      "/{tournament}/scripts",
      get(scoring::list_scripts).post(scoring::create_script),
    )
    .route("/{tournament}/scripts/templates", get(scoring::templates))
    .route(
      "/{tournament}/scripts/validate",
      post(scoring::validate_script),
    )
    .route(
      "/{tournament}/scripts/{script}/activate",
      post(scoring::activate_script),
    )
    .route(
      "/{tournament}/leaderboards/{kind}",
      get(scoring::leaderboard),
    )
    .route(
      "/{tournament}/leaderboards/recompute",
      post(scoring::recompute),
    )
}
