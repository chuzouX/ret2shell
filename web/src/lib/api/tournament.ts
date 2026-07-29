import type {
  ChartLibrary,
  ChartTag,
  LeaderboardSnapshot,
  Registration,
  ScoringScript,
  Tournament,
  TournamentChart,
  TournamentChartLibrary,
  TournamentResult,
  TournamentRound,
  TournamentStaff,
  TournamentTeam,
} from "@models/tournament";
import type { DateTime } from "luxon";
import api, { api_root, safeJson } from ".";

const root = `${api_root}/tournaments`;

export const getTournaments = async () => await api.get(root).json<Tournament[]>();
export const getTournament = async (id: number) => await api.get(`${root}/${id}`).json<Tournament>();
export const createTournament = async (input: Partial<Tournament> & Pick<Tournament, "name">) =>
  await api.post(root, { json: input }).json<Tournament>();
export const updateTournament = async (id: number, input: Partial<Tournament>) =>
  await api.patch(`${root}/${id}`, { json: input }).json<Tournament>();
export const deleteTournament = async (id: number) => await safeJson(api.delete(`${root}/${id}`).json<null>());
export const getStaff = async (id: number) => await api.get(`${root}/${id}/staff`).json<TournamentStaff[]>();
export const addStaff = async (id: number, user_id: number, role: TournamentStaff["role"]) =>
  await api.post(`${root}/${id}/staff`, { json: { user_id, role } }).json<TournamentStaff>();
export const removeStaff = async (id: number, user: number) =>
  await safeJson(api.delete(`${root}/${id}/staff/${user}`).json<null>());

export const getRounds = async (id: number) => await api.get(`${root}/${id}/rounds`).json<TournamentRound[]>();
export const createRound = async (id: number, input: Omit<TournamentRound, "id" | "tournament_id">) =>
  await api.post(`${root}/${id}/rounds`, { json: input }).json<TournamentRound>();
export const deleteRound = async (id: number, round: number) =>
  await safeJson(api.delete(`${root}/${id}/rounds/${round}`).json<null>());
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
export const getChartLibrary = async () => await api.get(`${api_root}/charts/library`).json<ChartLibrary[]>();
export const createLibraryChart = async (input: Omit<ChartLibrary, "id">) =>
  await api.post(`${api_root}/charts/library`, { json: input }).json<ChartLibrary>();
export const importPhiraChart = async (external_id: number) =>
  await api.post(`${api_root}/charts/library/import/phira`, { json: { external_id } }).json<ChartLibrary>();
export const getTournamentChartLibrary = async (id: number) =>
  await api.get(`${root}/${id}/chart-library`).json<TournamentChartLibrary[]>();
export interface TournamentChartLibraryInput {
  chart_library_id: number;
  round_id: number;
  tag_id: number;
  order_index: number;
  weight_millionths?: number;
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
