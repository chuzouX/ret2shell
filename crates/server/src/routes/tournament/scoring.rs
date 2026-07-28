use std::{
  collections::{HashMap, HashSet},
  hash::{DefaultHasher, Hash, Hasher},
  process::Stdio,
  time::Duration,
};

use axum::{
  Extension, Json,
  extract::{Path, Query, State},
  response::IntoResponse,
};
use chrono::Utc;
use r2s_database::{
  chart,
  leaderboard_snapshot::{self, LeaderboardKind},
  registration,
  result::{self, ResultStatus},
  scoring_script_version, team_member,
  tournament::{LeaderboardVisibility, Lifecycle},
  tournament_team,
};
use r2s_engine::Engine;
use r2s_migrator::Database;
use sea_orm::{
  ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
  QueryOrder, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::{io::AsyncWriteExt, process::Command};

use super::access;
use crate::{middleware::auth::Token, traits::ResponseError};

const BEST_PER_CHART: &str = "pub fn rank(context) { context.templates.best_per_chart }";
const ALL_RESULTS: &str = "pub fn rank(context) { context.templates.all_results }";
const RANK_POINTS: &str = "pub fn rank(context) { context.templates.rank_points }";
const WEIGHTED_TOP_N: &str = "pub fn rank(context) { context.templates.weighted_top_n }";

#[derive(Clone, Serialize)]
pub struct Template {
  key: &'static str,
  name: &'static str,
  source: &'static str,
}

fn builtin_templates() -> [Template; 4] {
  [
    Template {
      key: "best_per_chart",
      name: "Best score per chart",
      source: BEST_PER_CHART,
    },
    Template {
      key: "all_results",
      name: "All approved results",
      source: ALL_RESULTS,
    },
    Template {
      key: "rank_points",
      name: "Per-chart rank points",
      source: RANK_POINTS,
    },
    Template {
      key: "weighted_top_n",
      name: "Chart weights and Top-N members",
      source: WEIGHTED_TOP_N,
    },
  ]
}

pub async fn templates() -> impl IntoResponse {
  Json(builtin_templates())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ScriptEntry {
  id: i64,
  score: i64,
  #[serde(default)]
  tie_breakers: Vec<i64>,
  #[serde(default)]
  breakdown: Option<Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct ScriptOutput {
  #[serde(default)]
  individual: Vec<ScriptEntry>,
  #[serde(default)]
  team: Vec<ScriptEntry>,
}

#[derive(Clone, Serialize)]
struct RankedEntry {
  id: i64,
  name: String,
  rank: usize,
  score: i64,
  tie_breakers: Vec<i64>,
  breakdown: Option<Value>,
}

fn rank_entries(mut entries: Vec<ScriptEntry>, names: &HashMap<i64, String>) -> Vec<RankedEntry> {
  entries.sort_by(|left, right| {
    right
      .score
      .cmp(&left.score)
      .then_with(|| right.tie_breakers.cmp(&left.tie_breakers))
      .then_with(|| left.id.cmp(&right.id))
  });
  let mut previous: Option<(i64, Vec<i64>)> = None;
  let mut rank = 0;
  entries
    .into_iter()
    .enumerate()
    .map(|(index, entry)| {
      let signature = (entry.score, entry.tie_breakers.clone());
      if previous.as_ref() != Some(&signature) {
        rank = index + 1;
      }
      previous = Some(signature);
      RankedEntry {
        id: entry.id,
        name: names
          .get(&entry.id)
          .cloned()
          .unwrap_or_else(|| format!("#{}", entry.id)),
        rank,
        score: entry.score,
        tie_breakers: entry.tie_breakers,
        breakdown: entry.breakdown,
      }
    })
    .collect()
}

fn validate_output(
  output: &ScriptOutput, registration_ids: &HashSet<i64>, team_ids: &HashSet<i64>,
) -> Result<(), ResponseError> {
  for (label, rows, known) in [
    ("individual", &output.individual, registration_ids),
    ("team", &output.team, team_ids),
  ] {
    let mut seen = HashSet::new();
    for row in rows {
      if !known.contains(&row.id) {
        return Err(ResponseError::BadRequest(format!(
          "script returned unknown {label} id {}",
          row.id
        )));
      }
      if !seen.insert(row.id) {
        return Err(ResponseError::BadRequest(format!(
          "script returned duplicate {label} id {}",
          row.id
        )));
      }
      if row.tie_breakers.len() > 16 {
        return Err(ResponseError::BadRequest(
          "too many tie breakers".to_owned(),
        ));
      }
    }
  }
  Ok(())
}

fn aggregate_best(
  results: &[result::Model], weights: &HashMap<i64, i64>, weighted: bool,
) -> HashMap<i64, (i64, i64)> {
  let mut best: HashMap<(i64, i64), &result::Model> = HashMap::new();
  for row in results {
    best
      .entry((row.registration_id, row.chart_id))
      .and_modify(|current| {
        if (row.score, row.accuracy_millionths, -row.id)
          > (current.score, current.accuracy_millionths, -current.id)
        {
          *current = row;
        }
      })
      .or_insert(row);
  }
  let mut totals = HashMap::new();
  for ((registration_id, chart_id), row) in best {
    let score = if weighted {
      row
        .score
        .saturating_mul(*weights.get(&chart_id).unwrap_or(&1_000_000))
        / 1_000_000
    } else {
      row.score
    };
    let entry = totals.entry(registration_id).or_insert((0_i64, 0_i64));
    entry.0 = entry.0.saturating_add(score);
    entry.1 = entry.1.saturating_add(row.accuracy_millionths);
  }
  totals
}

fn entries_from_totals(totals: HashMap<i64, (i64, i64)>) -> Vec<ScriptEntry> {
  totals
    .into_iter()
    .map(|(id, (score, accuracy))| ScriptEntry {
      id,
      score,
      tie_breakers: vec![accuracy],
      breakdown: None,
    })
    .collect()
}

fn team_entries(
  individuals: &[ScriptEntry], members: &[team_member::Model], top_n: Option<usize>,
) -> Vec<ScriptEntry> {
  let registration_to_team: HashMap<i64, i64> = members
    .iter()
    .map(|m| (m.registration_id, m.team_id))
    .collect();
  let mut grouped: HashMap<i64, Vec<&ScriptEntry>> = HashMap::new();
  for row in individuals {
    if let Some(team_id) = registration_to_team.get(&row.id) {
      grouped.entry(*team_id).or_default().push(row);
    }
  }
  grouped
    .into_iter()
    .map(|(id, mut rows)| {
      rows.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
      if let Some(limit) = top_n {
        rows.truncate(limit);
      }
      ScriptEntry {
        id,
        score: rows
          .iter()
          .fold(0_i64, |total, row| total.saturating_add(row.score)),
        tie_breakers: vec![
          rows
            .iter()
            .flat_map(|row| row.tie_breakers.first())
            .copied()
            .sum(),
        ],
        breakdown: None,
      }
    })
    .collect()
}

fn builtins(
  results: &[result::Model], charts: &[chart::Model], members: &[team_member::Model], top_n: usize,
) -> Value {
  let weights: HashMap<i64, i64> = charts.iter().map(|c| (c.id, c.weight_millionths)).collect();
  let best_individual = entries_from_totals(aggregate_best(results, &weights, false));
  let weighted_individual = entries_from_totals(aggregate_best(results, &weights, true));
  let mut all_totals: HashMap<i64, (i64, i64)> = HashMap::new();
  for row in results {
    let entry = all_totals.entry(row.registration_id).or_default();
    entry.0 = entry.0.saturating_add(row.score);
    entry.1 = entry.1.saturating_add(row.accuracy_millionths);
  }
  let all_individual = entries_from_totals(all_totals);

  let mut per_chart: HashMap<i64, Vec<&result::Model>> = HashMap::new();
  let mut best_rows: HashMap<(i64, i64), &result::Model> = HashMap::new();
  for row in results {
    best_rows
      .entry((row.chart_id, row.registration_id))
      .and_modify(|old| {
        if (row.score, row.accuracy_millionths) > (old.score, old.accuracy_millionths) {
          *old = row;
        }
      })
      .or_insert(row);
  }
  for ((chart_id, _), row) in best_rows {
    per_chart.entry(chart_id).or_default().push(row);
  }
  let mut points: HashMap<i64, (i64, i64)> = HashMap::new();
  for rows in per_chart.values_mut() {
    rows.sort_by(|a, b| {
      b.score
        .cmp(&a.score)
        .then_with(|| b.accuracy_millionths.cmp(&a.accuracy_millionths))
        .then_with(|| a.id.cmp(&b.id))
    });
    let mut rank = 0;
    let mut previous = None;
    for (index, row) in rows.iter().enumerate() {
      let signature = (row.score, row.accuracy_millionths);
      if previous != Some(signature) {
        rank = index as i64 + 1;
      }
      previous = Some(signature);
      let entry = points.entry(row.registration_id).or_default();
      entry.0 += 0_i64.max(1000 - (rank - 1) * 50);
      entry.1 += row.score;
    }
  }
  let point_individual = entries_from_totals(points);
  let output = |individual: Vec<ScriptEntry>, limit: Option<usize>| ScriptOutput {
    team: team_entries(&individual, members, limit),
    individual,
  };
  json!({
    "best_per_chart": output(best_individual, None),
    "all_results": output(all_individual, None),
    "rank_points": output(point_individual, None),
    "weighted_top_n": output(weighted_individual, Some(top_n)),
  })
}

async fn context(
  db: &Database, tournament_id: i64,
) -> Result<(Value, HashSet<i64>, HashSet<i64>), ResponseError> {
  let tournament = access::tournament(db, tournament_id).await?;
  let registrations = registration::Entity::find()
    .filter(registration::Column::TournamentId.eq(tournament_id))
    .all(&db.conn)
    .await?;
  let teams = tournament_team::Entity::find()
    .filter(tournament_team::Column::TournamentId.eq(tournament_id))
    .all(&db.conn)
    .await?;
  let members = team_member::Entity::find()
    .filter(team_member::Column::TournamentId.eq(tournament_id))
    .all(&db.conn)
    .await?;
  let charts = chart::Entity::find()
    .filter(chart::Column::TournamentId.eq(tournament_id))
    .all(&db.conn)
    .await?;
  let results = result::Entity::find()
    .filter(result::Column::TournamentId.eq(tournament_id))
    .filter(result::Column::Status.eq(ResultStatus::Approved))
    .all(&db.conn)
    .await?;
  let templates = builtins(
    &results,
    &charts,
    &members,
    tournament.team_size_max.max(1) as usize,
  );
  let registration_ids = registrations.iter().map(|r| r.id).collect();
  let team_ids = teams.iter().map(|t| t.id).collect();
  Ok((
    json!({"tournament":tournament,"charts":charts,"registrations":registrations,"teams":teams,
    "team_members":members,"results":results,"templates":templates}),
    registration_ids,
    team_ids,
  ))
}

async fn run_script(
  db: &Database, tournament_id: i64, source: &str,
) -> Result<ScriptOutput, ResponseError> {
  if source.len() > 64 * 1024 {
    return Err(ResponseError::BadRequest(
      "script exceeds 64 KiB".to_owned(),
    ));
  }
  let diagnostics = Engine::lint_pure_json(source, &["rank"]).await?;
  if !diagnostics.is_empty() {
    return Err(ResponseError::BadRequest(serde_json::to_string(
      &diagnostics,
    )?));
  }
  let (context, registration_ids, team_ids) = context(db, tournament_id).await?;
  let json = serde_json::to_string(&context)?;
  let request = serde_json::to_vec(&json!({"source":source,"context":json}))?;
  let execution = async {
    let mut command = Command::new(std::env::current_exe()?);
    command
      .args(["internal", "score"])
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .kill_on_drop(true);
    let mut child = command.spawn()?;
    child
      .stdin
      .as_mut()
      .ok_or_else(|| std::io::Error::other("missing scoring process stdin"))?
      .write_all(&request)
      .await?;
    drop(child.stdin.take());
    child.wait_with_output().await
  };
  let process_output = tokio::time::timeout(Duration::from_secs(5), execution)
    .await
    .map_err(|_| ResponseError::BadRequest("scoring script exceeded 5 seconds".to_owned()))?
    .map_err(|error| {
      ResponseError::InternalServerError(format!("failed to run scoring process: {error}"))
    })?;
  if !process_output.status.success() {
    let error = String::from_utf8_lossy(&process_output.stderr)
      .trim()
      .to_owned();
    return Err(ResponseError::BadRequest(format!(
      "scoring process failed: {error}"
    )));
  }
  let output = String::from_utf8(process_output.stdout)?;
  let parsed: ScriptOutput = serde_json::from_str(&output)
    .map_err(|error| ResponseError::BadRequest(format!("invalid scoring output: {error}")))?;
  validate_output(&parsed, &registration_ids, &team_ids)?;
  Ok(parsed)
}

pub async fn list_scripts(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  Ok(Json(
    scoring_script_version::Entity::find()
      .filter(scoring_script_version::Column::TournamentId.eq(tournament_id))
      .order_by_desc(scoring_script_version::Column::Version)
      .all(&db.conn)
      .await?,
  ))
}

#[derive(Deserialize)]
pub struct ScriptInput {
  name: String,
  template_key: String,
  source: Option<String>,
}

fn template_source(key: &str) -> Option<&'static str> {
  builtin_templates()
    .into_iter()
    .find(|t| t.key == key)
    .map(|t| t.source)
}
fn source_hash(source: &str) -> String {
  let mut hash = DefaultHasher::new();
  source.hash(&mut hash);
  format!("{:016x}", hash.finish())
}

pub async fn create_script(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<ScriptInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let source = input
    .source
    .or_else(|| template_source(&input.template_key).map(str::to_owned))
    .ok_or_else(|| ResponseError::BadRequest("unknown scoring template".to_owned()))?;
  run_script(&db, tournament_id, &source).await?;
  let latest = scoring_script_version::Entity::find()
    .filter(scoring_script_version::Column::TournamentId.eq(tournament_id))
    .order_by_desc(scoring_script_version::Column::Version)
    .one(&db.conn)
    .await?;
  Ok(Json(
    scoring_script_version::ActiveModel {
      id: Default::default(),
      tournament_id: Set(tournament_id),
      version: Set(latest.map_or(1, |row| row.version + 1)),
      name: Set(input.name),
      template_key: Set(input.template_key),
      source_hash: Set(source_hash(&source)),
      source: Set(source),
      created_by: Set(token.id),
      created_at: Set(Utc::now()),
      active: Set(false),
      immutable: Set(true),
    }
    .insert(&db.conn)
    .await?,
  ))
}

#[derive(Deserialize)]
pub struct ValidateInput {
  source: String,
}
pub async fn validate_script(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
  Json(input): Json<ValidateInput>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let output = run_script(&db, tournament_id, &input.source).await?;
  Ok(Json(
    json!({"valid":true,"individual_entries":output.individual.len(),"team_entries":output.team.len()}),
  ))
}

pub async fn activate_script(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, script_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  let script = scoring_script_version::Entity::find_by_id(script_id)
    .one(&db.conn)
    .await?
    .filter(|s| s.tournament_id == tournament_id)
    .ok_or_else(|| ResponseError::NotFound("script version not found".to_owned()))?;
  run_script(&db, tournament_id, &script.source).await?;
  let txn = db.conn.begin().await?;
  let active = scoring_script_version::Entity::find()
    .filter(scoring_script_version::Column::TournamentId.eq(tournament_id))
    .filter(scoring_script_version::Column::Active.eq(true))
    .all(&txn)
    .await?;
  for row in active {
    let mut model = row.into_active_model();
    model.active = Set(false);
    model.update(&txn).await?;
  }
  let mut model = script.into_active_model();
  model.active = Set(true);
  let activated = model.update(&txn).await?;
  txn.commit().await?;
  recompute_now(&db, tournament_id).await?;
  Ok(Json(activated))
}

fn should_publish(visibility: LeaderboardVisibility, lifecycle: Lifecycle) -> bool {
  match visibility {
    LeaderboardVisibility::Live => true,
    LeaderboardVisibility::Frozen => false,
    LeaderboardVisibility::AfterEnd => {
      matches!(lifecycle, Lifecycle::Finished | Lifecycle::Archived)
    }
  }
}

pub(super) async fn recompute_now(db: &Database, tournament_id: i64) -> Result<(), ResponseError> {
  let tournament = access::tournament(db, tournament_id).await?;
  let script = scoring_script_version::Entity::find()
    .filter(scoring_script_version::Column::TournamentId.eq(tournament_id))
    .filter(scoring_script_version::Column::Active.eq(true))
    .one(&db.conn)
    .await?;
  let Some(script) = script else {
    return Ok(());
  };
  let public = should_publish(tournament.leaderboard_visibility, tournament.lifecycle);
  match run_script(db, tournament_id, &script.source).await {
    Ok(output) => {
      let registration_names = registration::Entity::find()
        .filter(registration::Column::TournamentId.eq(tournament_id))
        .all(&db.conn)
        .await?
        .into_iter()
        .map(|row| (row.id, row.display_name))
        .collect::<HashMap<_, _>>();
      let team_names = tournament_team::Entity::find()
        .filter(tournament_team::Column::TournamentId.eq(tournament_id))
        .all(&db.conn)
        .await?
        .into_iter()
        .map(|row| (row.id, row.name))
        .collect::<HashMap<_, _>>();
      let txn = db.conn.begin().await?;
      for (kind, entries) in [
        (
          LeaderboardKind::Individual,
          rank_entries(output.individual, &registration_names),
        ),
        (
          LeaderboardKind::Team,
          rank_entries(output.team, &team_names),
        ),
      ] {
        leaderboard_snapshot::ActiveModel {
          id: Default::default(),
          tournament_id: Set(tournament_id),
          script_version_id: Set(Some(script.id)),
          kind: Set(kind),
          entries: Set(serde_json::to_value(entries)?),
          stale: Set(false),
          error: Set(None),
          public_snapshot: Set(public),
          computed_at: Set(Utc::now()),
        }
        .insert(&txn)
        .await?;
      }
      txn.commit().await?;
    }
    Err(error) => {
      for kind in [LeaderboardKind::Individual, LeaderboardKind::Team] {
        let previous = leaderboard_snapshot::Entity::find()
          .filter(leaderboard_snapshot::Column::TournamentId.eq(tournament_id))
          .filter(leaderboard_snapshot::Column::Kind.eq(kind))
          .filter(leaderboard_snapshot::Column::Stale.eq(false))
          .order_by_desc(leaderboard_snapshot::Column::ComputedAt)
          .one(&db.conn)
          .await?;
        leaderboard_snapshot::ActiveModel {
          id: Default::default(),
          tournament_id: Set(tournament_id),
          script_version_id: Set(Some(script.id)),
          kind: Set(kind),
          entries: Set(previous.map(|p| p.entries).unwrap_or_else(|| json!([]))),
          stale: Set(true),
          error: Set(Some(error.to_string())),
          public_snapshot: Set(public),
          computed_at: Set(Utc::now()),
        }
        .insert(&db.conn)
        .await?;
      }
    }
  }
  Ok(())
}

pub async fn recompute(
  State(db): State<Database>, Extension(token): Extension<Token>, Path(tournament_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  access::require_organizer(&db, tournament_id, &token).await?;
  recompute_now(&db, tournament_id).await?;
  Ok(())
}

#[derive(Deserialize)]
pub struct LeaderboardQuery {
  staff_live: Option<bool>,
}
pub async fn leaderboard(
  State(db): State<Database>, Extension(token): Extension<Token>,
  Path((tournament_id, kind)): Path<(i64, String)>, Query(query): Query<LeaderboardQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  let tournament = access::tournament(&db, tournament_id).await?;
  let kind = match kind.as_str() {
    "individual" => LeaderboardKind::Individual,
    "team" => LeaderboardKind::Team,
    _ => {
      return Err(ResponseError::BadRequest(
        "leaderboard kind must be individual or team".to_owned(),
      ));
    }
  };
  let staff_live =
    query.staff_live.unwrap_or(false) && access::role(&db, tournament_id, &token).await?.is_some();
  let mut rows = leaderboard_snapshot::Entity::find()
    .filter(leaderboard_snapshot::Column::TournamentId.eq(tournament_id))
    .filter(leaderboard_snapshot::Column::Kind.eq(kind));
  if !staff_live && tournament.leaderboard_visibility != LeaderboardVisibility::Live {
    rows = rows.filter(leaderboard_snapshot::Column::PublicSnapshot.eq(true));
  }
  Ok(Json(
    rows
      .order_by_desc(leaderboard_snapshot::Column::ComputedAt)
      .one(&db.conn)
      .await?,
  ))
}

#[cfg(test)]
mod tests {
  use std::collections::{HashMap, HashSet};

  use chrono::Utc;
  use r2s_database::{chart, result, team_member};
  use serde_json::json;

  use super::{
    ScriptEntry, ScriptOutput, builtin_templates, builtins, rank_entries, validate_output,
  };

  fn chart(id: i64, weight_millionths: i64) -> chart::Model {
    chart::Model {
      id,
      tournament_id: 1,
      round_id: 1,
      tag_id: 1,
      title: format!("Chart {id}"),
      artist: "Artist".to_owned(),
      charter: "Charter".to_owned(),
      difficulty: "Master".to_owned(),
      level_constant: 12.0,
      cover: None,
      order_index: id as i32,
      weight_millionths,
      metadata: json!({}),
    }
  }

  fn result(
    id: i64, registration_id: i64, chart_id: i64, score: i64, accuracy_millionths: i64,
  ) -> result::Model {
    result::Model {
      id,
      tournament_id: 1,
      chart_id,
      registration_id,
      team_id_snapshot: Some(100),
      submitted_by: registration_id,
      score,
      accuracy_millionths,
      max_combo: 0,
      full_combo: false,
      all_perfect: false,
      judgments: json!({}),
      metrics: json!({}),
      played_at: Utc::now(),
      evidence: None,
      status: result::ResultStatus::Approved,
      replaces_result_id: None,
      created_at: Utc::now(),
    }
  }

  fn member(id: i64, registration_id: i64) -> team_member::Model {
    team_member::Model {
      id,
      tournament_id: 1,
      team_id: 100,
      registration_id,
      joined_at: Utc::now(),
    }
  }

  fn score(output: &ScriptOutput, registration_id: i64) -> i64 {
    output
      .individual
      .iter()
      .find(|entry| entry.id == registration_id)
      .unwrap()
      .score
  }

  #[test]
  fn templates_expose_rank_entrypoint() {
    for template in builtin_templates() {
      assert!(template.source.contains("rank(context)"));
    }
  }
  #[test]
  fn competition_ranking_skips_after_tie() {
    let ranked = rank_entries(
      vec![
        ScriptEntry {
          id: 1,
          score: 100,
          tie_breakers: vec![],
          breakdown: None,
        },
        ScriptEntry {
          id: 2,
          score: 90,
          tie_breakers: vec![],
          breakdown: None,
        },
        ScriptEntry {
          id: 3,
          score: 90,
          tie_breakers: vec![],
          breakdown: None,
        },
        ScriptEntry {
          id: 4,
          score: 80,
          tie_breakers: vec![],
          breakdown: None,
        },
      ],
      &HashMap::new(),
    );
    assert_eq!(
      ranked.iter().map(|r| r.rank).collect::<Vec<_>>(),
      vec![1, 2, 2, 4]
    );
  }

  #[test]
  fn builtin_templates_match_golden_scores() {
    let templates = builtins(
      &[
        result(1, 1, 10, 100, 900),
        result(2, 1, 10, 120, 850),
        result(3, 1, 11, 50, 990),
        result(4, 2, 10, 110, 950),
      ],
      &[chart(10, 2_000_000), chart(11, 500_000)],
      &[member(1, 1), member(2, 2)],
      1,
    );
    let output = |key| serde_json::from_value::<ScriptOutput>(templates[key].clone()).unwrap();

    let best = output("best_per_chart");
    assert_eq!((score(&best, 1), score(&best, 2)), (170, 110));
    assert_eq!(best.team[0].score, 280);

    let all = output("all_results");
    assert_eq!((score(&all, 1), score(&all, 2)), (270, 110));
    assert_eq!(all.team[0].score, 380);

    let points = output("rank_points");
    assert_eq!((score(&points, 1), score(&points, 2)), (2_000, 950));
    assert_eq!(points.team[0].score, 2_950);

    let weighted = output("weighted_top_n");
    assert_eq!((score(&weighted, 1), score(&weighted, 2)), (265, 220));
    assert_eq!(weighted.team[0].score, 265);
  }

  #[test]
  fn script_output_rejects_unknown_and_duplicate_ids() {
    let known_registrations = HashSet::from([1]);
    let known_teams = HashSet::from([100]);
    let entry = ScriptEntry {
      id: 2,
      score: 10,
      tie_breakers: vec![],
      breakdown: None,
    };
    let unknown = ScriptOutput {
      individual: vec![entry.clone()],
      team: vec![],
    };
    assert!(validate_output(&unknown, &known_registrations, &known_teams).is_err());

    let duplicate = ScriptOutput {
      individual: vec![
        ScriptEntry {
          id: 1,
          ..entry.clone()
        },
        ScriptEntry { id: 1, ..entry },
      ],
      team: vec![],
    };
    assert!(validate_output(&duplicate, &known_registrations, &known_teams).is_err());
  }
}
