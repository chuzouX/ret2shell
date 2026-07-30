import type { DateTime } from "luxon";

export type TournamentLifecycle = "draft" | "registration" | "running" | "review" | "finished" | "archived";
export type LifecycleScheduleMode = "manual" | "scheduled";
export type RoundReleaseAudience = "public" | "participants" | "staff";
export type RoundReleaseTiming = "immediate" | "on_enter" | "on_end";
export type RoundEndMode = "on_next_round" | "at_time" | "manual";
export type CompetitionMode = "individual" | "team" | "both";
export type EvidencePolicy = "required" | "optional" | "disabled";
export type LeaderboardVisibility = "live" | "frozen" | "after_end";
export type ChartVisibility = "public" | "after_archive" | "private";
export type ChartSourceType = "personal" | "phigros" | "phira";

export interface Tournament {
  id: number;
  name: string;
  brief: string;
  description?: string;
  owner_id: number;
  lifecycle: TournamentLifecycle;
  competition_mode: CompetitionMode;
  evidence_policy: EvidencePolicy;
  leaderboard_visibility: LeaderboardVisibility;
  cover?: string;
  team_size_min: number;
  team_size_max: number;
  registration_start_at?: DateTime;
  registration_end_at?: DateTime;
  start_at?: DateTime;
  end_at?: DateTime;
  registration_schedule: LifecycleScheduleMode;
  registration_at?: DateTime;
  running_schedule: LifecycleScheduleMode;
  running_at?: DateTime;
  review_schedule: LifecycleScheduleMode;
  review_at?: DateTime;
  finished_schedule: LifecycleScheduleMode;
  finished_at?: DateTime;
  organizer_can_edit_archived: boolean;
  current_round_id?: number;
  round_control_mode: "manual_assisted";
  created_at: DateTime;
  updated_at: DateTime;
}

export interface TournamentRound {
  id: number;
  tournament_id: number;
  name: string;
  description?: string;
  order_index: number;
  start_at?: DateTime;
  end_at?: DateTime;
  release_audience: RoundReleaseAudience[];
  release_timing: RoundReleaseTiming;
  end_mode: RoundEndMode;
  release_at?: DateTime;
  started_at?: DateTime;
  ended_at?: DateTime;
  released_at?: DateTime;
  manually_released: boolean;
  manually_ended: boolean;
  release_version: number;
}

export interface TournamentRoundConflict {
  current_round?: number;
  suggested_round: number;
  reason: string;
  affected_charts: number;
}
export interface ChartTag {
  id: number;
  tournament_id: number;
  round_id: number;
  name: string;
  order_index: number;
}
export interface TournamentChart {
  id: number;
  tournament_id: number;
  round_id: number;
  tag_id: number;
  title: string;
  artist: string;
  charter: string;
  difficulty: string;
  level_constant: number;
  cover?: string;
  description?: string;
  order_index: number;
  weight_millionths: number;
  metadata: Record<string, unknown>;
}
export interface ChartLibrary {
  id: number;
  title: string;
  artist: string;
  charter: string;
  difficulty: string;
  level_constant: number;
  weight_millionths?: number;
  cover?: string;
  metadata: Record<string, unknown>;
  source?: string;
  source_type?: ChartSourceType | string;
  tournaments?: string;
}
export interface TournamentChartLibrary {
  link: {
    id: number;
    tournament_id: number;
    chart_library_id: number | null;
    visibility: ChartVisibility;
    round_id: number;
    tag_id: number;
    order_index: number;
    weight_millionths: number;
    description?: string;
    title?: string;
    artist?: string;
    charter?: string;
    difficulty?: string;
    level_constant?: number;
    cover?: string;
    metadata?: Record<string, unknown>;
  };
  chart: ChartLibrary;
}
export interface Registration {
  id: number;
  tournament_id: number;
  user_id: number;
  display_name: string;
  status: "pending" | "approved" | "rejected" | "withdrawn";
  created_at: DateTime;
  updated_at: DateTime;
}
export interface TournamentTeam {
  id: number;
  tournament_id: number;
  name: string;
  captain_registration_id: number;
  created_at: DateTime;
}
export interface TournamentStaff {
  id: number;
  tournament_id: number;
  user_id: number;
  role: "owner" | "organizer" | "judge";
  created_at: DateTime;
}
export interface TournamentResult {
  id: number;
  tournament_id: number;
  chart_id: number;
  registration_id: number;
  team_id_snapshot?: number;
  submitted_by: number;
  score: number;
  accuracy_millionths: number;
  max_combo: number;
  full_combo: boolean;
  all_perfect: boolean;
  judgments: Record<string, number>;
  metrics: Record<string, unknown>;
  played_at: DateTime;
  evidence?: string;
  status: "pending" | "approved" | "rejected" | "voided";
  replaces_result_id?: number;
  created_at: DateTime;
}
export interface LeaderboardEntry {
  id: number;
  name: string;
  rank: number;
  score: number;
  tie_breakers: number[];
  breakdown?: unknown;
}
export interface LeaderboardSnapshot {
  id: number;
  tournament_id: number;
  kind: "individual" | "team";
  entries: LeaderboardEntry[];
  stale: boolean;
  error?: string;
  computed_at: DateTime;
}
export interface ScoringScript {
  id: number;
  tournament_id: number;
  version: number;
  name: string;
  template_key: string;
  source: string;
  source_hash: string;
  created_by: number;
  created_at: DateTime;
  active: boolean;
  immutable: boolean;
}
