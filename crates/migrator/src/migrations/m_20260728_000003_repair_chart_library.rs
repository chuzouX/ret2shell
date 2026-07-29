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
CREATE TABLE IF NOT EXISTS chart_source (
  id BIGSERIAL PRIMARY KEY,
  source_type VARCHAR(63) NOT NULL UNIQUE
);
INSERT INTO chart_source (source_type, name)
VALUES ('phira', 'Phira')
ON CONFLICT (source_type) DO UPDATE SET name = COALESCE(chart_source.name, 'Phira');
ALTER TABLE chart_library
  ADD COLUMN IF NOT EXISTS source_id BIGINT,
  ADD COLUMN IF NOT EXISTS external_id VARCHAR(255),
  ADD COLUMN IF NOT EXISTS exclusive_tournament_id BIGINT,
  ADD COLUMN IF NOT EXISTS created_by BIGINT,
  ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
UPDATE chart_library
SET source_id = (SELECT id FROM chart_source WHERE source_type = 'phira'),
    created_by = (SELECT id FROM "user" ORDER BY id LIMIT 1),
    created_at = CURRENT_TIMESTAMP,
    updated_at = CURRENT_TIMESTAMP
WHERE source_id IS NULL;
ALTER TABLE chart_library
  ALTER COLUMN source_id SET NOT NULL,
  ALTER COLUMN created_by SET NOT NULL,
  ALTER COLUMN created_at SET NOT NULL,
  ALTER COLUMN updated_at SET NOT NULL;
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chart_library_source_fk') THEN
    ALTER TABLE chart_library ADD CONSTRAINT chart_library_source_fk FOREIGN KEY (source_id) REFERENCES chart_source(id) ON DELETE RESTRICT;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chart_library_tournament_fk') THEN
    ALTER TABLE chart_library ADD CONSTRAINT chart_library_tournament_fk FOREIGN KEY (exclusive_tournament_id) REFERENCES tournament(id) ON DELETE SET NULL;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chart_library_creator_fk') THEN
    ALTER TABLE chart_library ADD CONSTRAINT chart_library_creator_fk FOREIGN KEY (created_by) REFERENCES "user"(id) ON DELETE RESTRICT;
  END IF;
END $$;
CREATE UNIQUE INDEX IF NOT EXISTS chart_library_source_external ON chart_library(source_id, external_id) WHERE external_id IS NOT NULL;
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
DROP INDEX chart_library_source_external;
ALTER TABLE chart_library
  DROP CONSTRAINT chart_library_creator_fk,
  DROP CONSTRAINT chart_library_tournament_fk,
  DROP CONSTRAINT chart_library_source_fk,
  DROP COLUMN updated_at,
  DROP COLUMN created_at,
  DROP COLUMN created_by,
  DROP COLUMN exclusive_tournament_id,
  DROP COLUMN external_id,
  DROP COLUMN source_id;
DROP TABLE chart_source;
"#,
      )
      .await?;
    Ok(())
  }
}
