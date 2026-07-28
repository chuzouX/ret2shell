import { handleHttpError } from "@api";
import { uploadMedia } from "@api/media";
import {
  activateScript,
  addStaff,
  commitImport,
  createChart,
  createChartTag,
  createRound,
  createScript,
  deleteChart,
  deleteChartTag,
  deleteRound,
  deleteTournament,
  getCharts,
  getChartTags,
  getRegistrations,
  getResults,
  getRounds,
  getScripts,
  getScriptTemplates,
  getStaff,
  getTournament,
  previewImport,
  type ResultInput,
  recomputeLeaderboard,
  removeStaff,
  reviewResult,
  updateTournament,
} from "@api/tournament";
import XLSX from "@e965/xlsx";
import type { CompetitionMode, EvidencePolicy } from "@models/tournament";
import { useParams } from "@solidjs/router";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Dialog from "@widgets/dialog";
import Input from "@widgets/input";
import Select from "@widgets/select";
import { DateTime } from "luxon";
import { createResource, createSignal, For, Match, Show, Switch } from "solid-js";

type Tab = "settings" | "staff" | "pool" | "scripts" | "review" | "import";

export default function () {
  const params = useParams();
  const id = () => Number(params.tournament);
  const [tab, setTab] = createSignal<Tab>("settings");
  const [tournament, { refetch: refetchTournament }] = createResource(id, getTournament);
  const [staff, { refetch: refetchStaff }] = createResource(id, getStaff);
  const [rounds, { refetch: refetchRounds }] = createResource(id, getRounds);
  const [chartTags, { refetch: refetchChartTags }] = createResource(id, getChartTags);
  const [charts, { refetch: refetchCharts }] = createResource(id, getCharts);
  const [scripts, { refetch: refetchScripts }] = createResource(id, getScripts);
  const [templates] = createResource(id, getScriptTemplates);
  const [results, { refetch: refetchResults }] = createResource(id, getResults);
  const [registrations] = createResource(id, getRegistrations);
  const [busy, setBusy] = createSignal(false);
  const [staffUser, setStaffUser] = createSignal("");
  const [staffRole, setStaffRole] = createSignal<"organizer" | "judge">("judge");
  const [roundName, setRoundName] = createSignal("");
  const [roundOrder, setRoundOrder] = createSignal("0");
  const [tagRound, setTagRound] = createSignal("");
  const [tagName, setTagName] = createSignal("");
  const [chartRound, setChartRound] = createSignal("");
  const [chartTag, setChartTag] = createSignal("");
  const [chartTitle, setChartTitle] = createSignal("");
  const [chartArtist, setChartArtist] = createSignal("");
  const [chartCharter, setChartCharter] = createSignal("");
  const [chartDifficulty, setChartDifficulty] = createSignal("");
  const [chartLevel, setChartLevel] = createSignal("0");
  const [chartWeight, setChartWeight] = createSignal("1.0");
  const [chartCover, setChartCover] = createSignal<File>();
  // Confirm dialog
  const [confirmOpen, setConfirmOpen] = createSignal(false);
  const [confirmMsg, setConfirmMsg] = createSignal("");
  const [confirmAction, setConfirmAction] = createSignal<() => Promise<unknown>>(() => Promise.resolve());
  const askConfirm = (msg: string, action: () => Promise<unknown>) => {
    setConfirmMsg(msg);
    setConfirmAction(() => action);
    setConfirmOpen(true);
  };
  const [scriptName, setScriptName] = createSignal("");
  const [templateKey, setTemplateKey] = createSignal("best_per_chart");
  const [source, setSource] = createSignal("");
  const [importRows, setImportRows] = createSignal<ResultInput[]>([]);
  const [preview, setPreview] = createSignal<Array<{ row: number; valid: boolean; error?: string }>>([]);
  // Basic info edit form
  const [editName, setEditName] = createSignal("");
  const [editBrief, setEditBrief] = createSignal("");
  const [editDescription, setEditDescription] = createSignal("");
  const [editMode, setEditMode] = createSignal<CompetitionMode>("both");
  const [editEvidence, setEditEvidence] = createSignal<EvidencePolicy>("optional");
  const [editTeamMin, setEditTeamMin] = createSignal("1");
  const [editTeamMax, setEditTeamMax] = createSignal("5");
  const [showEdit, setShowEdit] = createSignal(false);
  const toggleEdit = () => {
    const next = !showEdit();
    if (next) {
      const t = tournament();
      if (t) {
        setEditName(t.name);
        setEditBrief(t.brief);
        setEditDescription(t.description ?? "");
        setEditMode(t.competition_mode);
        setEditEvidence(t.evidence_policy);
        setEditTeamMin(String(t.team_size_min));
        setEditTeamMax(String(t.team_size_max));
      }
    }
    setShowEdit(next);
  };
  const saveEdit = () =>
    run(
      async () =>
        await updateTournament(id(), {
          name: editName().trim(),
          brief: editBrief().trim(),
          description: editDescription().trim() || undefined,
          competition_mode: editMode(),
          evidence_policy: editEvidence(),
          team_size_min: Math.max(1, Number(editTeamMin()) || 1),
          team_size_max: Math.max(1, Number(editTeamMax()) || 1),
        }),
      () => {
        refetchTournament();
        setShowEdit(false);
      }
    );

  const run = async (action: () => Promise<unknown>, refresh?: () => unknown) => {
    setBusy(true);
    try {
      await action();
      await refresh?.();
    } catch (error) {
      handleHttpError(error as Error, t("tournament.errors.action"));
    } finally {
      setBusy(false);
    }
  };
  const nextLifecycle = () =>
    (
      ({
        draft: "registration",
        registration: "running",
        running: "review",
        review: "finished",
        finished: "archived",
      }) as const
    )[tournament()?.lifecycle ?? "archived"];
  const chooseTemplate = (key: string) => {
    setTemplateKey(key);
    const value = templates()?.find((item) => item.key === key);
    if (value) {
      setSource(value.source);
      setScriptName(value.name);
    }
  };

  const parseImport = async (file?: File) => {
    if (!file) return;
    const workbook = XLSX.read(await file.arrayBuffer());
    const sheet = workbook.Sheets[workbook.SheetNames[0]];
    const raw = XLSX.utils.sheet_to_json<Record<string, unknown>>(sheet);
    const rows = raw.map((row) => ({
      registration_id: Number(row.registration_id),
      chart_id: Number(row.chart_id),
      score: Number(row.score),
      accuracy_millionths: Math.round(Number(row.accuracy) * 1_000_000),
      max_combo: Number(row.max_combo) || 0,
      full_combo: String(row.full_combo).toLowerCase() === "true",
      all_perfect: String(row.all_perfect).toLowerCase() === "true",
      judgments: {},
      metrics: {},
      played_at: row.played_at ? DateTime.fromISO(String(row.played_at)) : DateTime.now(),
      evidence: row.evidence ? String(row.evidence) : undefined,
    }));
    setImportRows(rows);
    setPreview(await previewImport(id(), rows));
  };

  const tabs: Array<[Tab, string, string]> = [
    ["settings", "icon-[fluent--options-20-regular]", "tournament.admin.settings"],
    ["staff", "icon-[fluent--people-settings-20-regular]", "tournament.admin.staff"],
    ["pool", "icon-[fluent--music-note-2-20-regular]", "tournament.admin.pool"],
    ["scripts", "icon-[fluent--code-20-regular]", "tournament.admin.scripts"],
    ["review", "icon-[fluent--clipboard-checkmark-20-regular]", "tournament.admin.review"],
    ["import", "icon-[fluent--table-arrow-up-20-regular]", "tournament.admin.import"],
  ];

  return (
    <main class="w-full max-w-6xl mx-auto px-4 py-8 space-y-6">
      <div>
        <h1 class="text-2xl font-bold">{t("tournament.admin.title")}</h1>
        <p class="opacity-60 mt-1">{t("tournament.admin.subtitle")}</p>
      </div>
      <div class="flex gap-2 overflow-x-auto pb-1">
        {tabs.map(([key, iconClass, label]) => (
          <Button size="sm" ghost={tab() !== key} active={tab() === key} onClick={() => setTab(key)}>
            <span class={`${iconClass} w-5 h-5`} />
            <span>{t(label)}</span>
          </Button>
        ))}
      </div>
      <Switch>
        <Match when={tab() === "settings"}>
          <section class="space-y-5">
            <div class="border border-layer-content/15 rounded-lg p-5 space-y-4">
              <div class="flex items-center gap-2">
                <span class="icon-[fluent--edit-20-regular] w-5 h-5 text-primary" />
                <h3 class="font-bold">{t("tournament.fields.basicInfo")}</h3>
                <span class="flex-1" />
                <Button size="sm" ghost onClick={toggleEdit}>
                  {showEdit() ? t("general.actions.close.title") : t("general.actions.edit.title")}
                </Button>
              </div>
              <Show when={showEdit()}>
                <div class="grid md:grid-cols-2 gap-3">
                  <Input
                    title={t("tournament.fields.name")}
                    value={editName()}
                    onInput={(e) => setEditName(e.currentTarget.value)}
                  />
                  <Input
                    title={t("tournament.fields.brief")}
                    value={editBrief()}
                    onInput={(e) => setEditBrief(e.currentTarget.value)}
                  />
                </div>
                <div class="flex flex-col space-y-1">
                  <span class="label">{t("tournament.fields.description")}</span>
                  <textarea
                    class="input input-md min-h-24 p-3"
                    value={editDescription()}
                    onInput={(e) => setEditDescription(e.currentTarget.value)}
                  />
                </div>
                <div class="grid sm:grid-cols-3 gap-3">
                  <Select
                    label={t("tournament.fields.mode")}
                    value={[editMode()]}
                    onValueChange={(e) => setEditMode(e.value[0] as CompetitionMode)}
                    items={[
                      { label: t("tournament.mode.individual"), value: "individual" },
                      { label: t("tournament.mode.team"), value: "team" },
                      { label: t("tournament.mode.both"), value: "both" },
                    ]}
                  />
                  <Select
                    label={t("tournament.fields.evidence")}
                    value={[editEvidence()]}
                    onValueChange={(e) => setEditEvidence(e.value[0] as EvidencePolicy)}
                    items={[
                      { label: t("tournament.evidence.required"), value: "required" },
                      { label: t("tournament.evidence.optional"), value: "optional" },
                      { label: t("tournament.evidence.disabled"), value: "disabled" },
                    ]}
                  />
                  <div class="flex gap-2 items-end">
                    <Input
                      title={t("tournament.fields.teamSizeMin")}
                      type="number"
                      size="sm"
                      min="1"
                      value={editTeamMin()}
                      onInput={(e) => setEditTeamMin(e.currentTarget.value)}
                    />
                    <Input
                      title={t("tournament.fields.teamSizeMax")}
                      type="number"
                      size="sm"
                      min="1"
                      value={editTeamMax()}
                      onInput={(e) => setEditTeamMax(e.currentTarget.value)}
                    />
                  </div>
                </div>
                <div class="flex justify-end">
                  <Button level="primary" loading={busy()} onClick={saveEdit}>
                    <span class="icon-[fluent--save-20-regular] w-5 h-5" />
                    {t("tournament.fields.save")}
                  </Button>
                </div>
              </Show>
            </div>
            <div class="grid md:grid-cols-3 gap-4 p-5 border border-layer-content/15 rounded-lg">
              <div>
                <div class="text-xs opacity-50">{t("tournament.fields.lifecycle")}</div>
                <div class="font-bold mt-1">{t(`tournament.lifecycle.${tournament()?.lifecycle}`)}</div>
              </div>
              <div>
                <div class="text-xs opacity-50">{t("tournament.fields.visibility")}</div>
                <div class="font-bold mt-1">{t(`tournament.visibility.${tournament()?.leaderboard_visibility}`)}</div>
              </div>
              <div class="flex items-center justify-end gap-2">
                <Button
                  disabled={!nextLifecycle()}
                  loading={busy()}
                  onClick={() =>
                    run(async () => await updateTournament(id(), { lifecycle: nextLifecycle() }), refetchTournament)
                  }
                >
                  {t("tournament.admin.advance")}
                </Button>
                <Button
                  level="primary"
                  onClick={() =>
                    run(async () => {
                      await recomputeLeaderboard(id());
                    }, refetchTournament)
                  }
                >
                  <span class="icon-[fluent--arrow-sync-20-regular] w-5 h-5" />
                  {t("tournament.admin.recompute")}
                </Button>
              </div>
            </div>
            <div class="flex flex-wrap gap-2">
              {(["live", "frozen", "after_end"] as const).map((visibility) => (
                <Button
                  ghost={tournament()?.leaderboard_visibility !== visibility}
                  active={tournament()?.leaderboard_visibility === visibility}
                  onClick={() =>
                    run(
                      async () => await updateTournament(id(), { leaderboard_visibility: visibility }),
                      refetchTournament
                    )
                  }
                >
                  {t(`tournament.visibility.${visibility}`)}
                </Button>
              ))}
            </div>
            <div class="border-t border-error/20 pt-4 mt-4">
              <Button
                level="error"
                ghost
                size="sm"
                onClick={() =>
                  askConfirm(`确定删除比赛 "${tournament()?.name}"？此操作不可撤销！`, async () => {
                    await deleteTournament(id());
                    window.location.href = "/tournaments";
                  })
                }
              >
                <span class="icon-[fluent--delete-20-regular] w-4 h-4" />
                {t("general.actions.delete.title")} 比赛
              </Button>
            </div>
          </section>
        </Match>
        <Match when={tab() === "staff"}>
          <section class="space-y-4">
            <div class="flex flex-wrap gap-2 items-end">
              <Input
                class="w-36"
                title={t("tournament.admin.userId")}
                type="number"
                value={staffUser()}
                onInput={(event) => setStaffUser(event.currentTarget.value)}
              />
              <Select
                label={t("tournament.admin.role")}
                value={[staffRole()]}
                onValueChange={(e) => setStaffRole(e.value[0] as "organizer" | "judge")}
                items={[
                  { label: t("tournament.roles.judge"), value: "judge" },
                  { label: t("tournament.roles.organizer"), value: "organizer" },
                ]}
              />
              <Button
                level="primary"
                disabled={!staffUser()}
                onClick={() => run(async () => await addStaff(id(), Number(staffUser()), staffRole()), refetchStaff)}
              >
                <span class="icon-[fluent--person-add-20-regular] w-5 h-5" />
                {t("general.actions.add.title")}
              </Button>
            </div>
            <div class="divide-y divide-layer-content/10 border border-layer-content/15 rounded-lg">
              <For each={staff()}>
                {(member) => (
                  <div class="p-3 flex items-center gap-3">
                    <span class="font-mono opacity-60">#{member.user_id}</span>
                    <strong>{t(`tournament.roles.${member.role}`)}</strong>
                    <span class="flex-1" />
                    <Show when={member.role !== "owner"}>
                      <Button
                        square
                        size="sm"
                        ghost
                        level="error"
                        title={t("general.actions.delete.title")}
                        onClick={() => run(async () => await removeStaff(id(), member.user_id), refetchStaff)}
                      >
                        <span class="icon-[fluent--delete-20-regular] w-5 h-5" />
                      </Button>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </section>
        </Match>
        <Match when={tab() === "pool"}>
          <section class="grid lg:grid-cols-2 gap-8">
            <div class="space-y-4">
              <h2 class="font-bold">{t("tournament.pool.rounds")}</h2>
              <div class="flex gap-2 items-end">
                <Input
                  size="sm"
                  noLabel
                  class="min-w-40"
                  placeholder={t("tournament.pool.roundName")}
                  value={roundName()}
                  onInput={(event) => setRoundName(event.currentTarget.value)}
                />
                <Input
                  size="sm"
                  noLabel
                  class="w-20"
                  type="number"
                  value={roundOrder()}
                  onInput={(event) => setRoundOrder(event.currentTarget.value)}
                />
                <Button
                  size="sm"
                  level="primary"
                  onClick={() =>
                    run(
                      async () =>
                        await createRound(id(), {
                          name: roundName(),
                          description: undefined,
                          order_index: Number(roundOrder()),
                          start_at: undefined,
                          end_at: undefined,
                        }),
                      refetchRounds
                    )
                  }
                >
                  {t("general.actions.add.title")}
                </Button>
              </div>
              <For each={rounds()}>
                {(round) => (
                  <div class="p-3 border border-layer-content/10 hover:border-layer-content/20 rounded-lg space-y-2 transition-colors bg-layer-content/[0.02]">
                    <div class="flex items-center gap-3">
                      <span class="font-mono text-primary text-sm font-bold w-6">{round.order_index}</span>
                      <strong class="flex-1">{round.name}</strong>
                      <Button
                        square
                        size="sm"
                        ghost
                        level="error"
                        title={t("general.actions.delete.title")}
                        onClick={() =>
                          askConfirm(`删除赛段 "${round.name}"？`, async () => {
                            await deleteRound(id(), round.id);
                            await refetchRounds();
                          })
                        }
                      >
                        <span class="icon-[fluent--delete-20-regular] w-4 h-4" />
                      </Button>
                    </div>
                    <div class="flex flex-wrap gap-2 pl-6">
                      <For
                        each={chartTags()?.filter((tag) => tag.round_id === round.id)}
                        fallback={<span class="text-xs opacity-50">{t("tournament.pool.noTags")}</span>}
                      >
                        {(tag) => (
                          <button
                            type="button"
                            class="inline-flex items-center gap-1 text-xs pl-2 pr-1 py-1 bg-layer-content/10 hover:bg-error/20 rounded transition-colors cursor-pointer group/tag"
                            title={t("general.actions.delete.title")}
                            onClick={() =>
                              askConfirm(`删除标签 "${tag.name}"？`, async () => {
                                await deleteChartTag(id(), tag.id);
                                await refetchChartTags();
                              })
                            }
                          >
                            <span class="icon-[fluent--tag-20-regular] w-3.5 h-3.5" />
                            {tag.name}
                            <span class="icon-[fluent--dismiss-12-regular] w-3 h-3 opacity-0 group-hover/tag:opacity-50" />
                          </button>
                        )}
                      </For>
                    </div>
                  </div>
                )}
              </For>
              <div class="border-t border-layer-content/15 pt-4 space-y-2">
                <h3 class="text-sm font-bold">{t("tournament.pool.tags")}</h3>
                <div class="grid sm:grid-cols-[1fr_1fr_auto] gap-2">
                  <Select
                    size="sm"
                    placeholder={t("tournament.pool.chooseRound")}
                    value={tagRound() ? [tagRound()] : []}
                    onValueChange={(e) => setTagRound(e.value[0] ?? "")}
                    items={(rounds() ?? []).map((r) => ({ label: r.name, value: String(r.id) }))}
                  />
                  <Input
                    size="sm"
                    noLabel
                    placeholder={t("tournament.pool.tagName")}
                    value={tagName()}
                    onInput={(event) => setTagName(event.currentTarget.value)}
                  />
                  <Button
                    size="sm"
                    level="primary"
                    disabled={!tagRound() || !tagName().trim()}
                    onClick={() =>
                      run(async () => {
                        await createChartTag(id(), {
                          round_id: Number(tagRound()),
                          name: tagName().trim(),
                          order_index: chartTags()?.filter((tag) => tag.round_id === Number(tagRound())).length ?? 0,
                        });
                        setTagName("");
                      }, refetchChartTags)
                    }
                  >
                    {t("general.actions.add.title")}
                  </Button>
                </div>
              </div>
            </div>
            <div class="space-y-4">
              <h2 class="font-bold">{t("tournament.pool.charts")}</h2>
              <div class="grid sm:grid-cols-2 gap-2">
                <Select
                  size="sm"
                  placeholder={t("tournament.pool.chooseRound")}
                  value={chartRound() ? [chartRound()] : []}
                  onValueChange={(e) => {
                    setChartRound(e.value[0] ?? "");
                    setChartTag("");
                  }}
                  items={(rounds() ?? []).map((r) => ({ label: r.name, value: String(r.id) }))}
                />
                <Select
                  size="sm"
                  placeholder={t("tournament.pool.chooseTag")}
                  disabled={!chartRound()}
                  value={chartTag() ? [chartTag()] : []}
                  onValueChange={(e) => setChartTag(e.value[0] ?? "")}
                  items={(chartTags()?.filter((tag) => tag.round_id === Number(chartRound())) ?? []).map((tag) => ({
                    label: tag.name,
                    value: String(tag.id),
                  }))}
                />
                <Input
                  size="sm"
                  noLabel
                  placeholder={t("tournament.pool.chartTitle")}
                  value={chartTitle()}
                  onInput={(event) => setChartTitle(event.currentTarget.value)}
                />
                <Input
                  size="sm"
                  noLabel
                  placeholder={t("tournament.pool.difficulty")}
                  value={chartDifficulty()}
                  onInput={(event) => setChartDifficulty(event.currentTarget.value)}
                />
                <Input
                  size="sm"
                  noLabel
                  placeholder={t("tournament.charts.charter")}
                  value={chartCharter()}
                  onInput={(event) => setChartCharter(event.currentTarget.value)}
                />
                <Input
                  size="sm"
                  noLabel
                  placeholder={t("tournament.charts.artist")}
                  value={chartArtist()}
                  onInput={(event) => setChartArtist(event.currentTarget.value)}
                />
                <Input
                  size="sm"
                  noLabel
                  type="number"
                  step="0.1"
                  placeholder={t("tournament.charts.constant")}
                  value={chartLevel()}
                  onInput={(event) => setChartLevel(event.currentTarget.value)}
                />
                <Input
                  size="sm"
                  noLabel
                  type="number"
                  step="0.01"
                  min="0"
                  placeholder={t("tournament.charts.weight")}
                  value={chartWeight()}
                  onInput={(event) => setChartWeight(event.currentTarget.value)}
                />
                <label class="btn btn-sm cursor-pointer">
                  <span class="icon-[fluent--image-add-20-regular] w-4 h-4" />
                  <span class="max-w-32 truncate">{chartCover()?.name || t("tournament.fields.cover")}</span>
                  <input
                    class="hidden"
                    type="file"
                    accept="image/*"
                    onChange={(e) => setChartCover(e.currentTarget.files?.[0])}
                  />
                </label>
                <Button
                  class="sm:col-span-2"
                  size="sm"
                  level="primary"
                  disabled={!chartRound() || !chartTag() || !chartTitle()}
                  onClick={() =>
                    run(
                      async () => {
                        let cover: string | undefined;
                        if (chartCover()) {
                          cover = (await uploadMedia(chartCover()!)).hash;
                        }
                        await createChart(id(), {
                          round_id: Number(chartRound()),
                          tag_id: Number(chartTag()),
                          title: chartTitle(),
                          artist: chartArtist() || "",
                          charter: chartCharter() || "",
                          difficulty: chartDifficulty(),
                          level_constant: Number(chartLevel()) || 0,
                          cover,
                          order_index: charts()?.filter((item) => item.tag_id === Number(chartTag())).length ?? 0,
                          weight_millionths: Math.round((Number(chartWeight()) || 1) * 1_000_000),
                          metadata: {},
                        });
                      },
                      () => {
                        refetchCharts();
                        setChartTitle("");
                        setChartDifficulty("");
                        setChartArtist("");
                        setChartCharter("");
                        setChartLevel("0");
                        setChartWeight("1.0");
                        setChartCover(undefined);
                      }
                    )
                  }
                >
                  {t("general.actions.add.title")}
                </Button>
              </div>
              <For each={charts()}>
                {(chart) => (
                  <div class="group p-3 border border-layer-content/10 hover:border-layer-content/20 rounded-lg transition-colors bg-layer-content/[0.02]">
                    <div class="flex items-center gap-3 mb-1.5">
                      <span class="font-mono text-xs text-primary font-bold shrink-0 min-w-10">{chart.difficulty}</span>
                      <strong class="flex-1 truncate">{chart.title}</strong>
                      <span class="text-xs opacity-50 px-2 py-0.5 rounded bg-layer-content/10 shrink-0">
                        {chartTags()?.find((tag) => tag.id === chart.tag_id)?.name}
                      </span>
                      <Button
                        square
                        size="sm"
                        ghost
                        level="error"
                        title={t("general.actions.delete.title")}
                        onClick={() =>
                          askConfirm(`删除谱面 "${chart.title}"？`, async () => {
                            await deleteChart(id(), chart.id);
                            await refetchCharts();
                          })
                        }
                      >
                        <span class="icon-[fluent--delete-20-regular] w-4 h-4" />
                      </Button>
                    </div>
                    <div class="flex items-center gap-3 text-xs opacity-40 pl-10">
                      <Show when={chart.artist}>
                        <span>{chart.artist}</span>
                      </Show>
                      <Show when={chart.charter}>
                        <span>{chart.charter}</span>
                      </Show>
                      <span class="font-mono">{chart.level_constant.toFixed(1)}</span>
                      <span class="font-mono opacity-60">×{(chart.weight_millionths / 1_000_000).toFixed(2)}</span>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </section>
        </Match>
        <Match when={tab() === "scripts"}>
          <section class="space-y-5">
            <div class="flex flex-wrap gap-2">
              <For each={templates()}>
                {(template) => (
                  <Button
                    size="sm"
                    ghost={templateKey() !== template.key}
                    active={templateKey() === template.key}
                    onClick={() => chooseTemplate(template.key)}
                  >
                    {template.name}
                  </Button>
                )}
              </For>
            </div>
            <Input
              title={t("tournament.admin.scriptName")}
              value={scriptName()}
              onInput={(event) => setScriptName(event.currentTarget.value)}
            />
            <textarea
              class="input w-full min-h-48 p-3 font-mono text-sm"
              value={source()}
              onInput={(event) => setSource(event.currentTarget.value)}
            />
            <Button
              level="primary"
              onClick={() =>
                run(
                  async () =>
                    await createScript(id(), { name: scriptName(), template_key: templateKey(), source: source() }),
                  refetchScripts
                )
              }
            >
              <span class="icon-[fluent--save-20-regular] w-5 h-5" />
              {t("tournament.admin.createVersion")}
            </Button>
            <div class="divide-y divide-layer-content/10 border border-layer-content/15 rounded-lg">
              <For each={scripts()}>
                {(script) => (
                  <div class="p-3 flex items-center gap-3">
                    <span class="font-mono opacity-60">v{script.version}</span>
                    <strong>{script.name}</strong>
                    <Show when={script.active}>
                      <span class="text-success text-xs">{t("tournament.admin.active")}</span>
                    </Show>
                    <span class="flex-1" />
                    <Button
                      size="sm"
                      disabled={script.active}
                      onClick={() => run(async () => await activateScript(id(), script.id), refetchScripts)}
                    >
                      {t("tournament.admin.activate")}
                    </Button>
                  </div>
                )}
              </For>
            </div>
          </section>
        </Match>
        <Match when={tab() === "review"}>
          <section class="border border-layer-content/15 rounded-lg overflow-x-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="bg-layer-content/5 text-left border-b border-layer-content/15">
                  <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider">
                    {t("tournament.registration.displayName")}
                  </th>
                  <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider">
                    {t("tournament.results.score")}
                  </th>
                  <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider">
                    {t("tournament.results.accuracy")}
                  </th>
                  <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider text-right">
                    {t("tournament.admin.review")}
                  </th>
                </tr>
              </thead>
              <tbody>
                <For
                  each={results()?.filter((item) => item.status === "pending")}
                  fallback={
                    <tr>
                      <td class="p-8" colSpan="4">
                        <div class="flex flex-col items-center justify-center gap-2 opacity-40">
                          <span class="icon-[fluent--clipboard-checkmark-20-regular] w-8 h-8" />
                          <span>{t("tournament.admin.noPending")}</span>
                        </div>
                      </td>
                    </tr>
                  }
                >
                  {(result) => (
                    <tr class="border-b border-layer-content/10 hover:bg-layer-content/5 transition-colors">
                      <td class="p-3">
                        {registrations()?.find((item) => item.id === result.registration_id)?.display_name}
                      </td>
                      <td class="p-3 font-mono tabular-nums">{result.score.toLocaleString()}</td>
                      <td class="p-3 font-mono tabular-nums">{(result.accuracy_millionths / 1_000_000).toFixed(4)}%</td>
                      <td class="p-3 flex justify-end gap-2">
                        <Button
                          size="sm"
                          level="success"
                          onClick={() =>
                            run(async () => await reviewResult(id(), result.id, "approved"), refetchResults)
                          }
                        >
                          {t("tournament.admin.approve")}
                        </Button>
                        <Button
                          size="sm"
                          level="error"
                          onClick={() =>
                            run(async () => await reviewResult(id(), result.id, "rejected"), refetchResults)
                          }
                        >
                          {t("tournament.admin.reject")}
                        </Button>
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </section>
        </Match>
        <Match when={tab() === "import"}>
          <section class="space-y-5">
            <label class="btn btn-md cursor-pointer w-fit">
              <span class="icon-[fluent--folder-open-20-regular] w-5 h-5" />
              {t("tournament.admin.selectFile")}
              <input
                class="hidden"
                type="file"
                accept=".csv,.xlsx,.xls"
                onChange={(event) => parseImport(event.currentTarget.files?.[0])}
              />
            </label>
            <Show when={preview().length}>
              <div class="border border-layer-content/15 rounded-lg divide-y divide-layer-content/10">
                <For each={preview()}>
                  {(row) => (
                    <div class="p-2 flex gap-3">
                      <span>#{row.row}</span>
                      <span class={row.valid ? "text-success" : "text-error"}>
                        {row.valid ? t("tournament.admin.valid") : row.error}
                      </span>
                    </div>
                  )}
                </For>
              </div>
              <Button
                level="primary"
                disabled={preview().some((row) => !row.valid)}
                onClick={() => run(async () => await commitImport(id(), importRows()), refetchResults)}
              >
                <span class="icon-[fluent--database-arrow-up-20-regular] w-5 h-5" />
                {t("tournament.admin.commitImport")}
              </Button>
            </Show>
          </section>
        </Match>
      </Switch>
      <Dialog
        open={confirmOpen()}
        onOpenChange={(e) => setConfirmOpen(e.open)}
        btnContent=""
        modal
        level="error"
        class="hidden"
      >
        <div class="space-y-4 min-w-[280px]">
          <p class="text-sm">{confirmMsg()}</p>
          <div class="flex justify-end gap-2">
            <Button size="sm" ghost onClick={() => setConfirmOpen(false)}>
              {t("general.actions.cancel.title")}
            </Button>
            <Button
              size="sm"
              level="error"
              onClick={() =>
                run(async () => {
                  await confirmAction()();
                  setConfirmOpen(false);
                })
              }
            >
              {t("general.actions.delete.title")}
            </Button>
          </div>
        </div>
      </Dialog>
    </main>
  );
}
