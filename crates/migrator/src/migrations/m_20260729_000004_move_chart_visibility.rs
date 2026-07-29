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
  ADD COLUMN IF NOT EXISTS visibility VARCHAR(16) NOT NULL DEFAULT 'private'
    CHECK (visibility IN ('public', 'after_archive', 'private'));
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = 'tournament' AND column_name = 'chart_visibility'
  ) THEN
    EXECUTE $migration$
      UPDATE tournament_chart_library tcl
      SET visibility = t.chart_visibility
      FROM tournament t
      WHERE t.id = tcl.tournament_id
        AND t.chart_visibility IN ('public', 'after_archive', 'private')
    $migration$;
    ALTER TABLE tournament DROP COLUMN chart_visibility;
  END IF;
END $$;
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
ALTER TABLE tournament
  ADD COLUMN IF NOT EXISTS chart_visibility VARCHAR(16) NOT NULL DEFAULT 'private'
    CHECK (chart_visibility IN ('public', 'after_archive', 'private'));
UPDATE tournament t
SET chart_visibility = COALESCE(
  (SELECT tcl.visibility FROM tournament_chart_library tcl
   WHERE tcl.tournament_id = t.id ORDER BY tcl.id LIMIT 1),
  'private'
);
ALTER TABLE tournament_chart_library DROP COLUMN IF EXISTS visibility;
"#,
      )
      .await?;
    Ok(())
  }
}
