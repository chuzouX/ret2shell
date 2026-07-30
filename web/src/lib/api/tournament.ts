import type {
  ChartLibrary,
  ChartTag,
  ChartVisibility,
  LifecycleScheduleMode,
  LeaderboardSnapshot,
  Registration,
  ScoringScript,
  Tournament,
  TournamentChart,
  TournamentChartLibrary,
  TournamentResult,
  TournamentRound,
  TournamentRoundConflict,
  TournamentStaff,
  TournamentTeam,
} from "@models/tournament";
import type { DateTime } from "luxon";
import api, { api_root, safeJson } from ".";

const root = `${api_root}/tournaments`;

export const getTournaments = async () => await api.get(root).json<Tournament[]>();
export const getTournament = async (id: number) => await api.get(`${root}/${id}`).json<Tournament>();
type CreateTournamentInput = Partial<Tournament> & Pick<Tournament, "name">;
export const createTournament = async (input: CreateTournamentInput) =>
  await api.post(root, { json: input }).json<Tournament>();
export type UpdateTournamentInput = Partial<Omit<Tournament, "registration_at" | "running_at" | "review_at" | "finished_at">> & {
  registration_at?: DateTime | null;
  running_at?: DateTime | null;
  review_at?: DateTime | null;
  finished_at?: DateTime | null;
  registration_schedule?: LifecycleScheduleMode;
  running_schedule?: LifecycleScheduleMode;
  review_schedule?: LifecycleScheduleMode;
  finished_schedule?: LifecycleScheduleMode;
};
export const updateTournament = async (id: number, input: UpdateTournamentInput) =>
  await api.patch(`${root}/${id}`, { json: input }).json<Tournament>();
export const deleteTournament = async (id: number) => await safeJson(api.delete(`${root}/${id}`).json<null>());
export const getStaff = async (id: number) => await api.get(`${root}/${id}/staff`).json<TournamentStaff[]>();
export const addStaff = async (id: number, user_id: number, role: TournamentStaff["role"]) =>
  await api.post(`${root}/${id}/staff`, { json: { user_id, role } }).json<TournamentStaff>();
export const removeStaff = async (id: number, user: number) =>
  await safeJson(api.delete(`${root}/${id}/staff/${user}`).json<null>());

export const getRounds = async (id: number) => await api.get(`${root}/${id}/rounds`).json<TournamentRound[]>();
export const createRound = async (id: number, input: RoundInput) =>
  await api.post(`${root}/${id}/rounds`, { json: input }).json<TournamentRound>();
export const deleteRound = async (id: number, round: number) =>
  await safeJson(api.delete(`${root}/${id}/rounds/${round}`).json<null>());
export interface RoundInput {
  name: string;
  description?: string;
  order_index: number;
  start_at?: DateTime | null;
  end_at?: DateTime | null;
  release_audience: Array<"public" | "participants" | "staff">;
  release_timing: "immediate" | "on_enter" | "on_end";
  end_mode: "on_next_round" | "at_time" | "manual";
  release_at?: DateTime | null;
}
export const updateRound = async (id: number, round: number, input: RoundInput) =>
  await api.patch(`${root}/${id}/rounds/${round}`, { json: input }).json<TournamentRound>();
export const enterRound = async (id: number, round: number, force = false) =>
  await api.post(`${root}/${id}/rounds/${round}/enter`, { json: { force } }).json<TournamentRound | TournamentRoundConflict>();
export const releaseRound = async (id: number, round: number) =>
  await api.post(`${root}/${id}/rounds/${round}/release`, { json: {} }).json<TournamentRound>();
export const withdrawRoundRelease = async (id: number, round: number) =>
  await api.post(`${root}/${id}/rounds/${round}/withdraw-release`, { json: {} }).json<TournamentRound>();
export const endRound = async (id: number, round: number) =>
  await api.post(`${root}/${id}/rounds/${round}/end`, { json: {} }).json<TournamentRound>();
export const getChartTags = async (id: number) => await api.get(`${root}/${id}/chart-tags`).json<ChartTag[]>();
export const createChartTag = async (id: number, input: Omit<ChartTag, "id" | "tournament_id">) =>
  await api.post(`${root}/${id}/chart-tags`, { json: input }).json<ChartTag>();
export const updateChartTag = async (id: number, tag: number, input: Omit<ChartTag, "id" | "tournament_id">) =>
  await api.patch(`${root}/${id}/chart-tags/${tag}`, { json: input }).json<ChartTag>();
export const deleteChartTag = async (id: number, tag: number) =>
  await safeJson(api.delete(`${root}/${id}/chart-tags/${tag}`).json<null>());
export const getCharts = async (id: number) => await api.get(`${root}/${id}/charts`).json<TournamentChart[]>();
export const createChart = async (id: number, input: Omit<TournamentChart, "id" | "tournament_id">) =>
  await api.post(`${root}/${id}/charts`, { json: input }).json<TournamentChart>();
export const deleteChart = async (id: number, chart: number) =>
  await safeJson(api.delete(`${root}/${id}/charts/${chart}`).json<null>());
type ChartLibraryListItem = {
  chart: ChartLibrary;
  source: string;
  source_type: string;
  tournaments: string;
};

export const getChartLibrary = async () => {
  const items = await api.get(`${api_root}/charts/library`).json<ChartLibraryListItem[]>();
  return items.map(({ chart, source, source_type, tournaments }) => ({
    ...chart,
    source,
    source_type,
    tournaments,
  }));
};
export interface ChartLibraryInput {
  source_type?: "personal" | "phigros" | "phira";
  title: string;
  artist: string;
  charter: string;
  difficulty: string;
  level_constant: number;
  cover?: string;
  metadata: Record<string, unknown>;
}
export const createLibraryChart = async (input: ChartLibraryInput) =>
  await api.post(`${api_root}/charts/library`, { json: input }).json<ChartLibrary>();
export const importPhiraChart = async (external_id: number) =>
  await api.post(`${api_root}/charts/library/import/phira`, { json: { external_id } }).json<ChartLibrary>();
export const getTournamentChartLibrary = async (id: number) =>
  await api.get(`${root}/${id}/chart-library`).json<TournamentChartLibrary[]>();
export interface TournamentChartLibraryInput {
  chart_library_id?: number | null;
  round_id: number;
  tag_id: number;
  order_index: number;
  weight_millionths?: number;
  description?: string;
  title?: string;
  artist?: string;
  charter?: string;
  difficulty?: string;
  level_constant?: number;
  cover?: string;
  metadata?: Record<string, unknown>;
  visibility: ChartVisibility;
}
export const addTournamentChartLibrary = async (id: number, input: TournamentChartLibraryInput) =>
  await api.post(`${root}/${id}/chart-library`, { json: input }).json<unknown>();
export const updateTournamentChartLibrary = async (id: number, link: number, input: TournamentChartLibraryInput) =>
  await api.patch(`${root}/${id}/chart-library/${link}`, { json: input }).json<unknown>();
export const removeTournamentChartLibrary = async (id: number, link: number) =>
  await safeJson(api.delete(`${root}/${id}/chart-library/${link}`).json<null>());

export const getMyRegistration = async (id: number) =>
  await api.get(`${root}/${id}/registrations/me`).json<Registration | null>();
export const registerTournament = async (id: number, display_name?: string) =>
  await api.post(`${root}/${id}/registrations`, { json: { display_name } }).json<Registration>();
export const withdrawRegistration = async (id: number) =>
  await safeJson(api.delete(`${root}/${id}/registrations/me`).json<null>());
export const getRegistrations = async (id: number) =>
  await api.get(`${root}/${id}/registrations`).json<Registration[]>();

export const getTeams = async (id: number) => await api.get(`${root}/${id}/teams`).json<TournamentTeam[]>();
export const createTeam = async (id: number, name: string) =>
  await api.post(`${root}/${id}/teams`, { json: { name } }).json<TournamentTeam>();
export const joinTeam = async (id: number, team: number) =>
  await api.post(`${root}/${id}/teams/${team}/join`).json<unknown>();
export const leaveTeam = async (id: number) => await safeJson(api.delete(`${root}/${id}/teams/leave`).json<null>());

export interface ResultInput {
  registration_id?: number;
  chart_id: number;
  score: number;
  accuracy_millionths: number;
  max_combo: number;
  full_combo: boolean;
  all_perfect: boolean;
  judgments: Record<string, number>;
  metrics: Record<string, unknown>;
  played_at: DateTime;
  evidence?: string;
}
export const getResults = async (id: number) => await api.get(`${root}/${id}/results`).json<TournamentResult[]>();
export const submitResult = async (id: number, input: ResultInput) =>
  await api.post(`${root}/${id}/results`, { json: input }).json<TournamentResult>();
export const previewImport = async (id: number, rows: ResultInput[]) =>
  await api
    .post(`${root}/${id}/results/import/preview`, { json: rows })
    .json<Array<{ row: number; valid: boolean; error?: string }>>();
export const commitImport = async (id: number, rows: ResultInput[]) =>
  await api.post(`${root}/${id}/results/import`, { json: rows }).json<TournamentResult[]>();
export const reviewResult = async (id: number, result: number, status: TournamentResult["status"], reason?: string) =>
  await api.post(`${root}/${id}/results/${result}/review`, { json: { status, reason } }).json<TournamentResult>();

export const getLeaderboard = async (id: number, kind: "individual" | "team", staff_live = false) =>
  await api
    .get(`${root}/${id}/leaderboards/${kind}`, { searchParams: { staff_live } })
    .json<LeaderboardSnapshot | null>();
export const getScripts = async (id: number) => await api.get(`${root}/${id}/scripts`).json<ScoringScript[]>();
export const getScriptTemplates = async (id: number) =>
  await api.get(`${root}/${id}/scripts/templates`).json<Array<{ key: string; name: string; source: string }>>();
export const createScript = async (id: number, input: { name: string; template_key: string; source?: string }) =>
  await api.post(`${root}/${id}/scripts`, { json: input }).json<ScoringScript>();
export const activateScript = async (id: number, script: number) =>
  await api.post(`${root}/${id}/scripts/${script}/activate`).json<ScoringScript>();
export const recomputeLeaderboard = async (id: number) =>
  await safeJson(api.post(`${root}/${id}/leaderboards/recompute`).json<null>());
