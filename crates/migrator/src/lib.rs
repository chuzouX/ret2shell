//! Used to init the new database and migrate the database from old versions.

use r2s_config::database;
use sea_orm::{ConnectOptions, DatabaseConnection};
use sea_orm_migration::prelude::*;
use tracing::log::LevelFilter;

mod migrations;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(migrations::m_20230101_000001_create_config::Migration),
      Box::new(migrations::m_20240101_000001_create_institute::Migration),
      Box::new(migrations::m_20240101_000002_create_user::Migration),
      Box::new(migrations::m_20240101_000003_create_oauth::Migration),
      Box::new(migrations::m_20240101_000004_create_ip::Migration),
      Box::new(migrations::m_20240101_000005_link_ip_user::Migration),
      Box::new(migrations::m_20240102_000001_create_article::Migration),
      Box::new(migrations::m_20240102_000002_create_comment::Migration),
      Box::new(migrations::m_20240102_000003_create_calendar::Migration),
      Box::new(migrations::m_20240103_000001_create_media::Migration),
      Box::new(migrations::m_20260727_000001_create_tournament_domain::Migration),
      Box::new(migrations::m_20260727_000002_create_chart_tags::Migration),
      Box::new(migrations::m_20260728_000001_add_chart_review::Migration),
      Box::new(migrations::m_20260728_000002_create_chart_library::Migration),
      Box::new(migrations::m_20260728_000003_repair_chart_library::Migration),
      Box::new(migrations::m_20260729_000001_expand_chart_library::Migration),
      Box::new(migrations::m_20260729_000002_expand_tournament_chart_library::Migration),
      Box::new(migrations::m_20260729_000004_move_chart_visibility::Migration),
      Box::new(migrations::m_20260729_000005_remove_exclusive_chart_source::Migration),
      Box::new(migrations::m_20260729_000006_add_phira_config::Migration),
      Box::new(migrations::m_20260729_000007_add_tournament_lifecycle_schedule::Migration),
      Box::new(migrations::m_20260730_000008_add_round_release_control::Migration),
      Box::new(migrations::m_20260731_000001_add_chart_library_status::Migration),
      Box::new(
        migrations::m_20260731_000002_add_tournament_rules_and_announcements_visibility::Migration,
      ),
      Box::new(migrations::m_20250105_000001_create_oauth_provider::Migration),
      Box::new(migrations::m_20250114_000001_create_oauth_index::Migration),
      Box::new(migrations::m_20250721_000001_create_ip_time_info::Migration),
    ]
  }
}

#[derive(Clone, Debug)]
pub struct Database {
  pub conn: DatabaseConnection,
}

async fn get_conn(config: &Option<database::Config>) -> Result<DatabaseConnection, DbErr> {
  let config = config
    .clone()
    .ok_or(DbErr::Custom("database config not found".to_string()))?;
  let mut connect_options = ConnectOptions::new(config.dsn());
  connect_options
    .acquire_timeout(std::time::Duration::from_secs(15))
    .sqlx_logging(true)
    .sqlx_logging_level(LevelFilter::Debug);
  sea_orm::Database::connect(connect_options).await
}

pub async fn initialize(config: &Option<database::Config>) -> Result<(Database, bool), DbErr> {
  let conn = get_conn(config).await?;
  let needs_migrate = !Migrator::get_pending_migrations(&conn).await?.is_empty();
  if needs_migrate {
    Migrator::up(&conn, None).await?;
  }
  Ok((Database { conn }, needs_migrate))
}

pub async fn down(config: &Option<database::Config>) -> Result<(), DbErr> {
  let conn = get_conn(config).await?;
  Migrator::down(&conn, None).await?;
  Ok(())
}
