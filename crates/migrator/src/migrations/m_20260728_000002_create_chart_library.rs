use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .get_connection()
      .execute_unprepared(
        r#"
CREATE TABLE chart_library (
  id BIGSERIAL PRIMARY KEY,
  title VARCHAR(255) NOT NULL,
  artist VARCHAR(255) NOT NULL DEFAULT '',
  charter VARCHAR(255) NOT NULL DEFAULT '',
  difficulty VARCHAR(63) NOT NULL,
  level_constant DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (level_constant >= 0),
  cover VARCHAR(255),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
CREATE TABLE tournament_chart_library (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  chart_library_id BIGINT NOT NULL REFERENCES chart_library(id) ON DELETE RESTRICT,
  visibility VARCHAR(16) NOT NULL DEFAULT 'private' CHECK (visibility IN ('public', 'after_archive', 'private')),
  round_id BIGINT NOT NULL REFERENCES tournament_round(id) ON DELETE CASCADE,
  tag_id BIGINT NOT NULL REFERENCES chart_tag(id) ON DELETE RESTRICT,
  order_index INTEGER NOT NULL DEFAULT 0,
  weight_millionths BIGINT NOT NULL DEFAULT 1000000 CHECK (weight_millionths >= 0),
  UNIQUE (tournament_id, chart_library_id),
  UNIQUE (tag_id, order_index)
);
CREATE INDEX tournament_chart_library_round ON tournament_chart_library(round_id, order_index);
"#,
      )
      .await?;
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .get_connection()
      .execute_unprepared(
        r#"
DROP TABLE tournament_chart_library;
DROP TABLE chart_library;
"#,
      )
      .await?;
    Ok(())
  }
}
