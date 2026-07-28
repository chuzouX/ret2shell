use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20260727_000001_create_tournament_domain"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(r#"
CREATE TABLE tournament (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(127) NOT NULL,
  brief VARCHAR(255) NOT NULL DEFAULT '',
  description TEXT,
  owner_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE RESTRICT ON UPDATE CASCADE,
  lifecycle VARCHAR(24) NOT NULL DEFAULT 'draft' CHECK (lifecycle IN ('draft','registration','running','review','finished','archived')),
  competition_mode VARCHAR(16) NOT NULL DEFAULT 'individual' CHECK (competition_mode IN ('individual','team','both')),
  evidence_policy VARCHAR(16) NOT NULL DEFAULT 'optional' CHECK (evidence_policy IN ('required','optional','disabled')),
  leaderboard_visibility VARCHAR(16) NOT NULL DEFAULT 'live' CHECK (leaderboard_visibility IN ('live','frozen','after_end')),
  cover VARCHAR(255),
  team_size_min INTEGER NOT NULL DEFAULT 1 CHECK (team_size_min > 0),
  team_size_max INTEGER NOT NULL DEFAULT 1 CHECK (team_size_max >= team_size_min),
  registration_start_at TIMESTAMPTZ,
  registration_end_at TIMESTAMPTZ,
  start_at TIMESTAMPTZ,
  end_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (registration_end_at IS NULL OR registration_start_at IS NULL OR registration_end_at > registration_start_at),
  CHECK (end_at IS NULL OR start_at IS NULL OR end_at > start_at)
);
CREATE TABLE tournament_staff (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  user_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
  role VARCHAR(16) NOT NULL CHECK (role IN ('owner','organizer','judge')),
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (tournament_id, user_id)
);
CREATE UNIQUE INDEX tournament_single_owner ON tournament_staff(tournament_id) WHERE role = 'owner';
CREATE TABLE tournament_round (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  name VARCHAR(127) NOT NULL,
  description TEXT,
  order_index INTEGER NOT NULL DEFAULT 0,
  start_at TIMESTAMPTZ,
  end_at TIMESTAMPTZ,
  UNIQUE (tournament_id, order_index),
  CHECK (end_at IS NULL OR start_at IS NULL OR end_at > start_at)
);
CREATE TABLE chart (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  round_id BIGINT NOT NULL REFERENCES tournament_round(id) ON DELETE CASCADE,
  title VARCHAR(255) NOT NULL,
  artist VARCHAR(255) NOT NULL DEFAULT '',
  charter VARCHAR(255) NOT NULL DEFAULT '',
  difficulty VARCHAR(63) NOT NULL,
  level_constant DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (level_constant >= 0),
  cover VARCHAR(255),
  order_index INTEGER NOT NULL DEFAULT 0,
  weight_millionths BIGINT NOT NULL DEFAULT 1000000 CHECK (weight_millionths >= 0),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (round_id, order_index)
);
CREATE TABLE registration (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  user_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
  display_name VARCHAR(127) NOT NULL,
  status VARCHAR(16) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','rejected','withdrawn')),
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (tournament_id, user_id)
);
CREATE TABLE tournament_team (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  name VARCHAR(127) NOT NULL,
  captain_registration_id BIGINT NOT NULL REFERENCES registration(id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (tournament_id, name)
);
CREATE TABLE team_member (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  team_id BIGINT NOT NULL REFERENCES tournament_team(id) ON DELETE CASCADE,
  registration_id BIGINT NOT NULL REFERENCES registration(id) ON DELETE CASCADE,
  joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (tournament_id, registration_id),
  UNIQUE (team_id, registration_id)
);
CREATE TABLE result (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  chart_id BIGINT NOT NULL REFERENCES chart(id) ON DELETE RESTRICT,
  registration_id BIGINT NOT NULL REFERENCES registration(id) ON DELETE RESTRICT,
  team_id_snapshot BIGINT REFERENCES tournament_team(id) ON DELETE SET NULL,
  submitted_by BIGINT NOT NULL REFERENCES "user"(id) ON DELETE RESTRICT,
  score BIGINT NOT NULL CHECK (score >= 0),
  accuracy_millionths BIGINT NOT NULL CHECK (accuracy_millionths BETWEEN 0 AND 100000000),
  max_combo INTEGER NOT NULL DEFAULT 0 CHECK (max_combo >= 0),
  full_combo BOOLEAN NOT NULL DEFAULT FALSE,
  all_perfect BOOLEAN NOT NULL DEFAULT FALSE,
  judgments JSONB NOT NULL DEFAULT '{}'::jsonb,
  metrics JSONB NOT NULL DEFAULT '{}'::jsonb,
  played_at TIMESTAMPTZ NOT NULL,
  evidence VARCHAR(255),
  status VARCHAR(16) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','rejected','voided')),
  replaces_result_id BIGINT UNIQUE REFERENCES result(id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (replaces_result_id IS NULL OR replaces_result_id <> id)
);
CREATE INDEX result_tournament_status ON result(tournament_id, status);
CREATE INDEX result_registration_chart ON result(registration_id, chart_id);
CREATE TABLE result_review (
  id BIGSERIAL PRIMARY KEY,
  result_id BIGINT NOT NULL REFERENCES result(id) ON DELETE CASCADE,
  reviewer_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE RESTRICT,
  from_status VARCHAR(16) NOT NULL CHECK (from_status IN ('pending','approved','rejected','voided')),
  to_status VARCHAR(16) NOT NULL CHECK (to_status IN ('pending','approved','rejected','voided')),
  reason TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (from_status <> to_status)
);
CREATE TABLE scoring_script_version (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  version INTEGER NOT NULL CHECK (version > 0),
  name VARCHAR(127) NOT NULL,
  template_key VARCHAR(63) NOT NULL,
  source TEXT NOT NULL CHECK (octet_length(source) <= 65536),
  source_hash VARCHAR(128) NOT NULL,
  created_by BIGINT NOT NULL REFERENCES "user"(id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  active BOOLEAN NOT NULL DEFAULT FALSE,
  immutable BOOLEAN NOT NULL DEFAULT TRUE,
  UNIQUE (tournament_id, version)
);
CREATE UNIQUE INDEX scoring_script_single_active ON scoring_script_version(tournament_id) WHERE active;
CREATE TABLE leaderboard_snapshot (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  script_version_id BIGINT REFERENCES scoring_script_version(id) ON DELETE SET NULL,
  kind VARCHAR(16) NOT NULL CHECK (kind IN ('individual','team')),
  entries JSONB NOT NULL DEFAULT '[]'::jsonb,
  stale BOOLEAN NOT NULL DEFAULT FALSE,
  error VARCHAR(1024),
  public_snapshot BOOLEAN NOT NULL DEFAULT FALSE,
  computed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX leaderboard_latest ON leaderboard_snapshot(tournament_id, kind, computed_at DESC);
CREATE INDEX leaderboard_public_latest ON leaderboard_snapshot(tournament_id, kind, public_snapshot, computed_at DESC);
CREATE TABLE notification (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  title VARCHAR(127) NOT NULL,
  content TEXT NOT NULL,
  publisher_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE RESTRICT,
  published_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX notification_tournament_published ON notification(tournament_id, published_at DESC);
CREATE TABLE chat (
  id BIGSERIAL PRIMARY KEY,
  tournament_id BIGINT NOT NULL REFERENCES tournament(id) ON DELETE CASCADE,
  user_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE RESTRICT,
  content TEXT NOT NULL CHECK (char_length(content) BETWEEN 1 AND 4000),
  is_staff BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX chat_tournament_created ON chat(tournament_id, created_at DESC);
    "#).await?;
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(r#"
DROP TABLE IF EXISTS chat, notification, leaderboard_snapshot, scoring_script_version, result_review, result,
  team_member, tournament_team, registration, chart, tournament_round, tournament_staff,
  tournament CASCADE;
    "#).await?;
    Ok(())
  }
}
