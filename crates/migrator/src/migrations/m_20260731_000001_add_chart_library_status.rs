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
  ADD COLUMN IF NOT EXISTS status VARCHAR(16) NOT NULL DEFAULT 'approved'
  CHECK (status IN ('pending', 'approved', 'rejected'));
"#,
      )
      .await?;
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .get_connection()
      .execute_unprepared("ALTER TABLE chart_library DROP COLUMN status;")
      .await?;
    Ok(())
  }
}
