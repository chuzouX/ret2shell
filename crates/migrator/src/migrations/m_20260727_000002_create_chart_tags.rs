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
CREATE TABLE chart_tag (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  round_id BIGINT NOT NULL REFERENCES tournament_round(id) ON DELETE CASCADE,
  name VARCHAR(127) NOT NULL,
  order_index INTEGER NOT NULL DEFAULT 0,
  UNIQUE (round_id, name),
  UNIQUE (round_id, order_index)
);
INSERT INTO chart_tag (tournament_id, round_id, name, order_index)
SELECT tournament_id, id, 'General', 0 FROM tournament_round;
ALTER TABLE chart ADD COLUMN tag_id BIGINT;
UPDATE chart
SET tag_id = chart_tag.id
FROM chart_tag
WHERE chart.round_id = chart_tag.round_id AND chart_tag.order_index = 0;
ALTER TABLE chart ALTER COLUMN tag_id SET NOT NULL;
ALTER TABLE chart
  ADD CONSTRAINT chart_tag_fk FOREIGN KEY (tag_id) REFERENCES chart_tag(id) ON DELETE RESTRICT;
ALTER TABLE chart DROP CONSTRAINT chart_round_id_order_index_key;
ALTER TABLE chart ADD CONSTRAINT chart_tag_order_unique UNIQUE (tag_id, order_index);
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
ALTER TABLE chart DROP CONSTRAINT chart_tag_fk;
ALTER TABLE chart DROP CONSTRAINT chart_tag_order_unique;
UPDATE chart
SET order_index = ranked.new_order
FROM (
  SELECT id, (ROW_NUMBER() OVER (PARTITION BY round_id ORDER BY tag_id, order_index, id) - 1)::INTEGER AS new_order
  FROM chart
) AS ranked
WHERE chart.id = ranked.id;
ALTER TABLE chart DROP COLUMN tag_id;
ALTER TABLE chart ADD CONSTRAINT chart_round_id_order_index_key UNIQUE (round_id, order_index);
DROP TABLE chart_tag;
"#,
      )
      .await?;
    Ok(())
  }
}
