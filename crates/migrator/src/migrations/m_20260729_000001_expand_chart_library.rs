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
ALTER TABLE chart_library
  ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE chart_source
  ADD COLUMN IF NOT EXISTS name VARCHAR(255);
UPDATE chart_source SET name = source_type WHERE name IS NULL;
ALTER TABLE chart_source ALTER COLUMN name SET NOT NULL;
INSERT INTO chart_source (source_type, name)
VALUES
  ('personal', '个人上传'),
  ('phigros', 'Phigros'),
  ('phira', 'Phira')
ON CONFLICT (source_type) DO UPDATE SET name = EXCLUDED.name;
"#,
      )
      .await?;
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .get_connection()
      .execute_unprepared("ALTER TABLE chart_library DROP COLUMN description;")
      .await?;
    Ok(())
  }
}
