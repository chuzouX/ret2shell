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
UPDATE chart_library
SET source_id = (SELECT id FROM chart_source WHERE source_type = 'personal' LIMIT 1)
WHERE source_id IN (SELECT id FROM chart_source WHERE source_type = 'exclusive');
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'chart_library' AND column_name = 'exclusive_tournament_id') THEN
    ALTER TABLE chart_library DROP CONSTRAINT IF EXISTS chart_library_tournament_fk;
    ALTER TABLE chart_library DROP COLUMN exclusive_tournament_id;
  END IF;
END $$;
DELETE FROM chart_source WHERE source_type = 'exclusive';
"#,
      )
      .await?;
    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Ok(())
  }
}
