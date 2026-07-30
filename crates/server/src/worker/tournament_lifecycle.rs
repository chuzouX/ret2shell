use chrono::Utc;
use r2s_database::tournament::{self, Lifecycle, LifecycleScheduleMode};
use r2s_migrator::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::time::{Duration, interval};
use tracing::{error, info};

use crate::routes::tournament::scoring;

pub fn spawn(db: Database) {
  tokio::spawn(async move {
    let mut ticker = interval(Duration::from_secs(15));
    loop {
      ticker.tick().await;
      if let Err(error) = advance_due(&db).await {
        error!(?error, "failed to advance scheduled tournament lifecycle");
      }
    }
  });
}

async fn advance_due(db: &Database) -> Result<(), sea_orm::DbErr> {
  let now = Utc::now();
  let rows = tournament::Entity::find()
    .filter(tournament::Column::Lifecycle.ne(Lifecycle::Archived))
    .all(&db.conn)
    .await?;

  for current in rows {
    let mut lifecycle = current.lifecycle;
    loop {
      let Some((next, schedule, at)) = next_schedule(&current, lifecycle) else {
        break;
      };
      if schedule != LifecycleScheduleMode::Scheduled || at.is_none_or(|value| value > now) {
        break;
      }

      let result = tournament::Entity::update_many()
        .col_expr(tournament::Column::Lifecycle, next.into())
        .col_expr(tournament::Column::UpdatedAt, now.into())
        .filter(tournament::Column::Id.eq(current.id))
        .filter(tournament::Column::Lifecycle.eq(lifecycle))
        .exec(&db.conn)
        .await?;
      if result.rows_affected == 0 {
        break;
      }
      scoring::recompute_now(db, current.id)
        .await
        .map_err(|error| {
          sea_orm::DbErr::Custom(format!("failed to recompute leaderboard: {error}"))
        })?;
      info!(
        tournament_id = current.id,
        ?next,
        "scheduled tournament lifecycle advanced"
      );
      lifecycle = next;
    }
  }
  Ok(())
}

fn next_schedule(
  row: &tournament::Model, lifecycle: Lifecycle,
) -> Option<(
  Lifecycle,
  LifecycleScheduleMode,
  Option<chrono::DateTime<Utc>>,
)> {
  match lifecycle {
    Lifecycle::Draft => Some((
      Lifecycle::Registration,
      row.registration_schedule,
      row.registration_at,
    )),
    Lifecycle::Registration => Some((Lifecycle::Running, row.running_schedule, row.running_at)),
    Lifecycle::Running => Some((Lifecycle::Review, row.review_schedule, row.review_at)),
    Lifecycle::Review => Some((Lifecycle::Finished, row.finished_schedule, row.finished_at)),
    Lifecycle::Finished | Lifecycle::Archived => None,
  }
}
