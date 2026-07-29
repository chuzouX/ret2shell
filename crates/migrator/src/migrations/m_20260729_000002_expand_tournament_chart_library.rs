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
ALTER TABLE tournament_chart_library
  ALTER COLUMN chart_library_id DROP NOT NULL,
  ADD COLUMN IF NOT EXISTS description TEXT,
  ADD COLUMN IF NOT EXISTS title VARCHAR(255) NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS artist VARCHAR(255) NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS charter VARCHAR(255) NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS difficulty VARCHAR(63) NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS level_constant DOUBLE PRECISION NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS cover VARCHAR(255),
  ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb;
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
ALTER TABLE tournament_chart_library
  DROP COLUMN IF EXISTS metadata,
  DROP COLUMN IF EXISTS cover,
  DROP COLUMN IF EXISTS level_constant,
  DROP COLUMN IF EXISTS difficulty,
  DROP COLUMN IF EXISTS charter,
  DROP COLUMN IF EXISTS artist,
  DROP COLUMN IF EXISTS title,
  DROP COLUMN IF EXISTS description;
ALTER TABLE tournament_chart_library ALTER COLUMN chart_library_id SET NOT NULL;
"#,
      )
      .await?;
    Ok(())
  }
}
