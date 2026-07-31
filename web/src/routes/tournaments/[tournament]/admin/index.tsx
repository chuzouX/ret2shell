import { handleHttpError } from "@api";
import { uploadMedia } from "@api/media";
import {
  activateScript,
  addStaff,
  addTournamentChartLibrary,
  commitImport,
  createChartTag,
  createRound,
  createScript,
  deleteChart,
  deleteChartTag,
  deleteRound,
  deleteTournament,
  deleteNotification,
  endRound,
  enterRound,
  getChartLibrary,
  getCharts,
  getChartTags,
  getNotifications,
  getRegistrations,
  getResults,
  getRounds,
  getScripts,
  getScriptTemplates,
  getStaff,
  getTournament,
  getTournamentChartLibrary,
  importPhiraChart,
  previewImport,
  publishNotification,
  type ResultInput,
  recomputeLeaderboard,
  releaseRound,
  removeStaff,
  removeTournamentChartLibrary,
  reviewResult,
  updateTournament,
  updateTournamentChartLibrary,
  withdrawRoundRelease,
} from "@api/tournament";
import XLSX from "@e965/xlsx";
import type {
  ChartSourceType,
  ChartVisibility,
  CompetitionMode,
  EvidencePolicy,
  LifecycleScheduleMode,
  RoundEndMode,
  RoundReleaseAudience,
  RoundReleaseTiming,
  TournamentLifecycle,
} from "@models/tournament";
import { Permission } from "@models/user";
import { useParams } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Dialog from "@widgets/dialog";
import Input from "@widgets/input";
import Select from "@widgets/select";
import { DateTime } from "luxon";
import { createResource, createSignal, For, Match, Show, Switch } from "solid-js";

type Tab = "settings" | "staff" | "rounds" | "charts" | "scripts" | "review" | "import" | "content";
type ChartImportMethod = "custom" | "library" | "phira";

export default function () {
  const params = useParams();
  const id = () => Number(params.tournament);
  const [tab, setTab] = createSignal<Tab>("settings");
  const [tournament, { refetch: refetchTournament }] = createResource(id, getTournament);
  const [staff, { refetch: refetchStaff }] = createResource(id, getStaff);
  const [rounds, { refetch: refetchRounds }] = createResource(id, getRounds);
  const [chartTags, { refetch: refetchChartTags }] = createResource(id, getChartTags);
  const [charts, { refetch: refetchCharts }] = createResource(id, getCharts);
  const [chartLibrary] = createResource(getChartLibrary);
  const [tournamentChartLibrary, { refetch: refetchTournamentChartLibrary }] = createResource(
    id,
    getTournamentChartLibrary
  );
  const [scripts, { refetch: refetchScripts }] = createResource(id, getScripts);
  const [templates] = createResource(id, getScriptTemplates);
  const [results, { refetch: refetchResults }] = createResource(id, getResults);
  const [registrations] = createResource(id, getRegistrations);
  const [notifications, { refetch: refetchNotifications }] = createResource(id, getNotifications);
  const [busy, setBusy] = createSignal(false);
  const [staffUser, setStaffUser] = createSignal("");
  const [staffRole, setStaffRole] = createSignal<"organizer" | "judge">("judge");
  const [roundName, setRoundName] = createSignal("");
  const [roundOrder, setRoundOrder] = createSignal("0");
  const [roundReleaseTiming, setRoundReleaseTiming] = createSignal<RoundReleaseTiming>("on_enter");
  const [roundEndMode, setRoundEndMode] = createSignal<RoundEndMode>("on_next_round");
  const [roundAudience, setRoundAudience] = createSignal<RoundReleaseAudience[]>(["staff"]);
  const [roundReleaseAt, setRoundReleaseAt] = createSignal("");
  const [tagRound, setTagRound] = createSignal("");
  const [tagName, setTagName] = createSignal("");
  const [libraryRound, setLibraryRound] = createSignal("");
  const [libraryTag, setLibraryTag] = createSignal("");
  const [librarySelection, setLibrarySelection] = createSignal("");
  const [libraryWeight, setLibraryWeight] = createSignal("1.0");
  const [phiraId, setPhiraId] = createSignal("");
  const [libraryTitle, setLibraryTitle] = createSignal("");
  const [libraryArtist, setLibraryArtist] = createSignal("");
  const [libraryCharter, setLibraryCharter] = createSignal("");
  const [libraryDifficulty, setLibraryDifficulty] = createSignal("");
  const [libraryLevel, setLibraryLevel] = createSignal("0");
  const [showLibraryAdd, setShowLibraryAdd] = createSignal(true);
  const [chartImportMethod, setChartImportMethod] = createSignal<ChartImportMethod>("custom");
  const [chartSource, setChartSource] = createSignal<ChartSourceType>("personal");
  const [chartDescription, setChartDescription] = createSignal("");
  const [chartCover, setChartCover] = createSignal<File>();
  const [libraryCover, setLibraryCover] = createSignal("");
  const [metadataLocked, setMetadataLocked] = createSignal(false);
  const [phiraDialogOpen, setPhiraDialogOpen] = createSignal(false);
  const [confirmOpen, setConfirmOpen] = createSignal(false);
  const [confirmMsg, setConfirmMsg] = createSignal("");
  const [confirmActionLabel, setConfirmActionLabel] = createSignal(t("general.actions.delete.title"));
  const [confirmAction, setConfirmAction] = createSignal<() => Promise<unknown>>(() => Promise.resolve());
  const askConfirm = (msg: string, action: () => Promise<unknown>, actionLabel = t("general.actions.delete.title")) => {
    setConfirmMsg(msg);
    setConfirmActionLabel(actionLabel);
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
  const [libraryVisibility, setLibraryVisibility] = createSignal<ChartVisibility>();
  const [editTeamMin, setEditTeamMin] = createSignal("1");
  const [editTeamMax, setEditTeamMax] = createSignal("5");
  const [editLifecycle, setEditLifecycle] = createSignal<TournamentLifecycle>("draft");
  const [registrationSchedule, setRegistrationSchedule] = createSignal<LifecycleScheduleMode>("manual");
  const [registrationAt, setRegistrationAt] = createSignal("");
  const [runningSchedule, setRunningSchedule] = createSignal<LifecycleScheduleMode>("manual");
  const [runningAt, setRunningAt] = createSignal("");
  const [reviewSchedule, setReviewSchedule] = createSignal<LifecycleScheduleMode>("manual");
  const [reviewAt, setReviewAt] = createSignal("");
  const [finishedSchedule, setFinishedSchedule] = createSignal<LifecycleScheduleMode>("manual");
  const [finishedAt, setFinishedAt] = createSignal("");
  const [organizerCanEditArchived, setOrganizerCanEditArchived] = createSignal(false);
  const [showEdit, setShowEdit] = createSignal(false);
  const [showLifecycleEdit, setShowLifecycleEdit] = createSignal(false);
  const [editRules, setEditRules] = createSignal("");
  const [notifTitle, setNotifTitle] = createSignal("");
  const [notifContent, setNotifContent] = createSignal("");
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
      async () => {
        const current = tournament();
        if (!current) return;
        await updateTournament(id(), {
          name: editName().trim(),
          brief: editBrief().trim(),
          description: editDescription().trim() || undefined,
          competition_mode: editMode(),
          evidence_policy: editEvidence(),
          team_size_min: Math.max(1, Number(editTeamMin()) || 1),
          team_size_max: Math.max(1, Number(editTeamMax()) || 1),
        });
      },
      () => {
        refetchTournament();
        setShowEdit(false);
      }
    );

  const openLifecycleEdit = () => {
    const current = tournament();
    if (!current) return;
    setEditLifecycle(current.lifecycle);
    setRegistrationSchedule(current.registration_schedule);
    setRegistrationAt(toLocalDateTime(current.registration_at));
    setRunningSchedule(current.running_schedule);
    setRunningAt(toLocalDateTime(current.running_at));
    setReviewSchedule(current.review_schedule);
    setReviewAt(toLocalDateTime(current.review_at));
    setFinishedSchedule(current.finished_schedule);
    setFinishedAt(toLocalDateTime(current.finished_at));
    setOrganizerCanEditArchived(current.organizer_can_edit_archived);
    setShowLifecycleEdit((value) => !value);
  };

  const saveLifecycle = () =>
    run(
      async () => {
        const current = tournament();
        if (!current) return;
        const stages = [
          [registrationSchedule(), registrationAt()],
          [runningSchedule(), runningAt()],
          [reviewSchedule(), reviewAt()],
          [finishedSchedule(), finishedAt()],
        ] as const;
        let previous: DateTime | undefined;
        for (const [mode, value] of stages) {
          const at = mode === "scheduled" && value ? DateTime.fromISO(value) : undefined;
          if (mode === "scheduled" && !at?.isValid) {
            throw new Error(t("tournament.admin.stageStartRequired"));
          }
          if (at && !at.isValid) {
            throw new Error(t("tournament.admin.invalidStageStart"));
          }
          if (at && at < current.created_at) {
            throw new Error(t("tournament.admin.stageStartBeforeCreation"));
          }
          if (at && previous && at < previous) {
            throw new Error(t("tournament.admin.stageStartOrder"));
          }
          if (at) previous = at;
        }
        await updateTournament(id(), {
          lifecycle: editLifecycle(),
          registration_schedule: registrationSchedule(),
          registration_at: stageTime(registrationSchedule(), registrationAt()),
          running_schedule: runningSchedule(),
          running_at: stageTime(runningSchedule(), runningAt()),
          review_schedule: reviewSchedule(),
          review_at: stageTime(reviewSchedule(), reviewAt()),
          finished_schedule: finishedSchedule(),
          finished_at: stageTime(finishedSchedule(), finishedAt()),
          organizer_can_edit_archived: organizerCanEditArchived(),
        });
      },
      () => {
        refetchTournament();
        setShowLifecycleEdit(false);
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
  const selectTab = (key: Tab) => {
    if (key === "content") {
      setEditRules(tournament()?.rules ?? "");
    }
    setTab(key);
  };
  const saveRules = () =>
    run(async () => {
      await updateTournament(id(), { rules: editRules().trim() || undefined });
    }, refetchTournament);
  const toggleRulesVisible = (value: boolean) =>
    run(async () => await updateTournament(id(), { rules_visible: value }), refetchTournament);
  const toggleAnnouncementsVisible = (value: boolean) =>
    run(async () => await updateTournament(id(), { announcements_visible: value }), refetchTournament);
  const publishAnnouncement = () =>
    run(async () => {
      if (!notifTitle().trim() || !notifContent().trim()) return;
      await publishNotification(id(), { title: notifTitle().trim(), content: notifContent().trim() });
      setNotifTitle("");
      setNotifContent("");
    }, refetchNotifications);
  const removeNotification = (notification: number) =>
    askConfirm(t("general.actions.delete.message"), async () => {
      await deleteNotification(id(), notification);
      await refetchNotifications();
    });
  const lifecycleItems = ["draft", "registration", "running", "review", "finished", "archived"] as const;
  const scheduleItems = ["manual", "scheduled"] as const;
  const toLocalDateTime = (value?: DateTime) => value?.toLocal().toFormat("yyyy-MM-dd'T'HH:mm") || "";
  const fromLocalDateTime = (value: string) => (value ? DateTime.fromISO(value).toUTC() : null);
  const stageTime = (mode: LifecycleScheduleMode, value: string) =>
    mode === "scheduled" ? fromLocalDateTime(value) : null;
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
  const requestLifecycleChange = (next: TournamentLifecycle) => {
    const current = tournament();
    if (!current || current.lifecycle === next) return;
    askConfirm(
      `${t(`tournament.lifecycle.${current.lifecycle}`)} → ${t(`tournament.lifecycle.${next}`)}`,
      async () => {
        await updateTournament(id(), { lifecycle: next });
        await refetchTournament();
      },
      t("tournament.admin.confirm")
    );
  };
  const chooseTemplate = (key: string) => {
    setTemplateKey(key);
    const value = templates()?.find((item) => item.key === key);
    if (value) {
      setSource(value.source);
      setScriptName(value.name);
    }
  };
  const selectLibraryChart = (chartId: string) => {
    const chart = chartLibrary()?.find((item) => item.id === Number(chartId));
    setLibrarySelection(chartId);
    if (!chart) return;
    setLibraryTitle(chart.title);
    setLibraryArtist(chart.artist);
    setLibraryCharter(chart.charter);
    setLibraryDifficulty(chart.difficulty);
    setLibraryLevel(String(chart.level_constant));
    setLibraryCover(chart.cover ?? "");
    setChartDescription("");
    setMetadataLocked(true);
  };
  const resetPersonalForm = () => {
    setLibrarySelection("");
    setLibraryTitle("");
    setLibraryArtist("");
    setLibraryCharter("");
    setLibraryDifficulty("");
    setLibraryLevel("0");
    setLibraryCover("");
    setChartDescription("");
    setMetadataLocked(false);
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
    ["content", "icon-[fluent--document-bullet-20-regular]", "tournament.admin.content"],
    ["staff", "icon-[fluent--people-settings-20-regular]", "tournament.admin.staff"],
    ["rounds", "icon-[fluent--calendar-agenda-20-regular]", "tournament.admin.rounds"],
    ["charts", "icon-[fluent--music-note-2-20-regular]", "tournament.admin.charts"],
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
          <Button size="sm" ghost={tab() !== key} active={tab() === key} onClick={() => selectTab(key)}>
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
                  <Select
                    label={t("tournament.admin.releaseTiming")}
                    value={[roundReleaseTiming()]}
                    items={[
                      { label: t("tournament.admin.releaseTimingOptions.immediate"), value: "immediate" },
                      { label: t("tournament.admin.releaseTimingOptions.onEnter"), value: "on_enter" },
                      { label: t("tournament.admin.releaseTimingOptions.onEnd"), value: "on_end" },
                    ]}
                    onValueChange={(event) => setRoundReleaseTiming(event.value[0] as RoundReleaseTiming)}
                  />
                  <Select
                    label={t("tournament.admin.roundEndMode")}
                    value={[roundEndMode()]}
                    items={[
                      { label: t("tournament.admin.roundEndOptions.onNextRound"), value: "on_next_round" },
                      { label: t("tournament.admin.roundEndOptions.atTime"), value: "at_time" },
                      { label: t("tournament.admin.roundEndOptions.manual"), value: "manual" },
                    ]}
                    onValueChange={(event) => setRoundEndMode(event.value[0] as RoundEndMode)}
                  />
                  <Show when={roundEndMode() === "at_time"}>
                    <Input
                      title={t("tournament.admin.roundEndAt")}
                      type="datetime-local"
                      value={roundReleaseAt()}
                      onInput={(event) => setRoundReleaseAt(event.currentTarget.value)}
                    />
                  </Show>
                  <div class="sm:col-span-2 border border-layer-content/15 rounded-lg p-3 space-y-2">
                    <div class="label">{t("tournament.admin.releaseAudience")}</div>
                    <div class="flex flex-wrap gap-3 text-sm">
                      {(["public", "participants", "staff"] as const).map((audience) => (
                        <label class="flex items-center gap-2">
                          <input
                            type="checkbox"
                            checked={roundAudience().includes(audience)}
                            onChange={(event) => {
                              const values = roundAudience();
                              setRoundAudience(
                                event.currentTarget.checked
                                  ? [...values, audience]
                                  : values.filter((value) => value !== audience)
                              );
                            }}
                          />
                          {t(`tournament.admin.releaseAudienceOptions.${audience}`)}
                        </label>
                      ))}
                    </div>
                  </div>
                </div>
                <div class="flex flex-col space-y-1">
                  <span class="label">{t("tournament.fields.description")}</span>
                  <textarea
                    class="input input-md min-h-24 p-3"
                    value={editDescription()}
                    onInput={(e) => setEditDescription(e.currentTarget.value)}
                  />
                </div>
                <div class="grid sm:grid-cols-2 gap-3">
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
                  <Show when={editMode() !== "individual"}>
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
                  </Show>
                </div>
                <div class="flex justify-end">
                  <Button level="primary" loading={busy()} onClick={saveEdit}>
                    <span class="icon-[fluent--save-20-regular] w-5 h-5" />
                    {t("tournament.fields.save")}
                  </Button>
                </div>
              </Show>
            </div>
            <div class="border border-primary/25 rounded-lg p-5 space-y-5">
              <div class="flex items-start gap-3">
                <span class="icon-[fluent--arrow-sync-circle-20-regular] w-6 h-6 text-primary" />
                <div class="min-w-0">
                  <h3 class="font-bold">{t("tournament.admin.lifecycleControl")}</h3>
                  <p class="text-sm opacity-60 mt-1">{t("tournament.admin.lifecycleControlDescription")}</p>
                </div>
                <span class="flex-1" />
                <Button size="sm" ghost onClick={openLifecycleEdit}>
                  <span class="icon-[fluent--settings-20-regular] w-4 h-4" />
                  {showLifecycleEdit() ? t("general.actions.close.title") : t("tournament.admin.lifecycleSettings")}
                </Button>
              </div>
              <div class="flex flex-wrap items-center gap-2">
                <div class="px-3 py-2 rounded-md bg-primary/15 text-primary font-bold">
                  {t(`tournament.lifecycle.${tournament()?.lifecycle}`)}
                </div>
                <span class="icon-[fluent--arrow-right-20-regular] w-5 h-5 opacity-40" />
                <Show
                  when={nextLifecycle()}
                  fallback={<span class="text-sm opacity-60">{t("tournament.admin.lifecycleComplete")}</span>}
                >
                  {(next) => (
                    <Button level="primary" loading={busy()} onClick={() => requestLifecycleChange(next())}>
                      <span class="icon-[fluent--arrow-forward-20-regular] w-5 h-5" />
                      {t("tournament.admin.advanceTo", { stage: t(`tournament.lifecycle.${next()}`) })}
                    </Button>
                  )}
                </Show>
              </div>
              <div class="grid grid-cols-2 md:grid-cols-6 gap-2">
                {lifecycleItems.map((stage, index) => (
                  <div
                    class="relative border rounded-md p-3 text-center"
                    classList={{
                      "border-primary bg-primary/10": tournament()?.lifecycle === stage,
                      "border-layer-content/15 opacity-50": tournament()?.lifecycle !== stage,
                    }}
                  >
                    <div class="text-xs opacity-60">{index + 1}</div>
                    <div class="font-bold text-sm mt-1">{t(`tournament.lifecycle.${stage}`)}</div>
                  </div>
                ))}
              </div>
              <Show when={showLifecycleEdit()}>
                <div class="border-t border-layer-content/15 pt-4 space-y-4">
                  <div class="flex items-center gap-2">
                    <span class="icon-[fluent--calendar-clock-20-regular] w-5 h-5 text-primary" />
                    <h4 class="font-bold">{t("tournament.admin.lifecycleSettings")}</h4>
                  </div>
                  <Select
                    label={t("tournament.fields.lifecycle")}
                    value={[editLifecycle()]}
                    disabled={
                      tournament()?.lifecycle === "archived" && !accountStore.permissions.includes(Permission.DevOps)
                    }
                    onValueChange={(e) => setEditLifecycle(e.value[0] as TournamentLifecycle)}
                    items={lifecycleItems.map((value) => ({
                      label: t(`tournament.lifecycle.${value}`),
                      value,
                    }))}
                  />
                  <div class="grid md:grid-cols-2 gap-3">
                    <For
                      each={
                        [
                          [
                            "registration",
                            registrationSchedule,
                            setRegistrationSchedule,
                            registrationAt,
                            setRegistrationAt,
                          ],
                          ["running", runningSchedule, setRunningSchedule, runningAt, setRunningAt],
                          ["review", reviewSchedule, setReviewSchedule, reviewAt, setReviewAt],
                          ["finished", finishedSchedule, setFinishedSchedule, finishedAt, setFinishedAt],
                        ] as const
                      }
                    >
                      {(stage) => (
                        <div class="border border-layer-content/15 rounded-lg p-3 space-y-2">
                          <div class="font-bold">{t(`tournament.lifecycle.${stage[0]}`)}</div>
                          <Select
                            label={t("tournament.admin.scheduleMode")}
                            value={[stage[1]()]}
                            items={scheduleItems.map((value) => ({
                              label: t(`tournament.admin.schedule.${value}`),
                              value,
                            }))}
                            onValueChange={(e) => stage[2](e.value[0] as LifecycleScheduleMode)}
                          />
                          <Input
                            title={t("tournament.admin.stageStart")}
                            type="datetime-local"
                            value={stage[3]()}
                            disabled={stage[1]() !== "scheduled"}
                            onInput={(e) => stage[4](e.currentTarget.value)}
                          />
                        </div>
                      )}
                    </For>
                  </div>
                  <Show when={accountStore.permissions.includes(Permission.DevOps)}>
                    <label class="flex items-center gap-2">
                      <input
                        type="checkbox"
                        checked={organizerCanEditArchived()}
                        onChange={(e) => setOrganizerCanEditArchived(e.currentTarget.checked)}
                      />
                      <span>{t("tournament.admin.organizerCanEditArchived")}</span>
                    </label>
                  </Show>
                  <div class="flex justify-end gap-2">
                    <Button size="sm" ghost onClick={() => setShowLifecycleEdit(false)}>
                      {t("general.actions.cancel.title")}
                    </Button>
                    <Button level="primary" loading={busy()} onClick={saveLifecycle}>
                      <span class="icon-[fluent--save-20-regular] w-5 h-5" />
                      {t("tournament.admin.saveLifecycle")}
                    </Button>
                  </div>
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
        <Match when={tab() === "content"}>
          <section class="space-y-5">
            <div class="border border-layer-content/15 rounded-lg p-5 space-y-4">
              <div class="flex items-center gap-2">
                <span class="icon-[fluent--document-text-20-regular] w-5 h-5 text-primary" />
                <h3 class="font-bold">{t("tournament.rules.title")}</h3>
                <span class="flex-1" />
                <label class="flex items-center gap-2 text-sm">
                  <input
                    type="checkbox"
                    checked={tournament()?.rules_visible ?? false}
                    onChange={(event) => toggleRulesVisible(event.currentTarget.checked)}
                  />
                  {t("tournament.admin.showRulesTab")}
                </label>
              </div>
              <div class="flex flex-col space-y-1">
                <span class="label">{t("tournament.fields.description")}</span>
                <textarea
                  class="input input-md min-h-48 p-3"
                  value={editRules()}
                  onInput={(event) => setEditRules(event.currentTarget.value)}
                />
              </div>
              <div class="flex justify-end">
                <Button level="primary" loading={busy()} onClick={saveRules}>
                  <span class="icon-[fluent--save-20-regular] w-5 h-5" />
                  {t("tournament.fields.save")}
                </Button>
              </div>
            </div>
            <div class="border border-layer-content/15 rounded-lg p-5 space-y-4">
              <div class="flex items-center gap-2">
                <span class="icon-[fluent--megaphone-loud-20-regular] w-5 h-5 text-primary" />
                <h3 class="font-bold">{t("tournament.announcements.title")}</h3>
                <span class="flex-1" />
                <label class="flex items-center gap-2 text-sm">
                  <input
                    type="checkbox"
                    checked={tournament()?.announcements_visible ?? false}
                    onChange={(event) => toggleAnnouncementsVisible(event.currentTarget.checked)}
                  />
                  {t("tournament.admin.showAnnouncementsTab")}
                </label>
              </div>
              <div class="grid md:grid-cols-2 gap-3">
                <Input
                  title={t("tournament.announcements.titleField")}
                  value={notifTitle()}
                  onInput={(event) => setNotifTitle(event.currentTarget.value)}
                />
              </div>
              <div class="flex flex-col space-y-1">
                <span class="label">{t("tournament.announcements.contentField")}</span>
                <textarea
                  class="input input-md min-h-32 p-3"
                  value={notifContent()}
                  onInput={(event) => setNotifContent(event.currentTarget.value)}
                />
              </div>
              <div class="flex justify-end">
                <Button
                  level="primary"
                  loading={busy()}
                  disabled={!notifTitle().trim() || !notifContent().trim()}
                  onClick={publishAnnouncement}
                >
                  <span class="icon-[fluent--send-20-regular] w-5 h-5" />
                  {t("tournament.admin.publishAnnouncement")}
                </Button>
              </div>
              <div class="divide-y divide-layer-content/10 border border-layer-content/15 rounded-lg">
                <For
                  each={notifications()}
                  fallback={
                    <div class="p-8 flex flex-col items-center justify-center gap-2 opacity-40">
                      <span class="icon-[fluent--megaphone-loud-20-regular] w-8 h-8" />
                      <span class="text-sm">{t("tournament.announcements.empty")}</span>
                    </div>
                  }
                >
                  {(item) => (
                    <div class="p-3 flex items-start gap-3">
                      <div class="flex-1 min-w-0 space-y-1">
                        <div class="flex items-center gap-2">
                          <strong class="truncate">{item.title}</strong>
                          <span class="text-xs opacity-50 shrink-0">
                            {item.published_at?.toLocal().toFormat("yyyy-MM-dd HH:mm")}
                          </span>
                        </div>
                        <p class="text-sm opacity-70 whitespace-pre-wrap break-words">{item.content}</p>
                      </div>
                      <Button
                        square
                        size="sm"
                        ghost
                        level="error"
                        title={t("general.actions.delete.title")}
                        onClick={() => removeNotification(item.id)}
                      >
                        <span class="icon-[fluent--delete-20-regular] w-5 h-5" />
                      </Button>
                    </div>
                  )}
                </For>
              </div>
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
        <Match when={tab() === "rounds" || tab() === "charts"}>
          <section class="grid lg:grid-cols-[minmax(260px,0.8fr)_minmax(0,1.6fr)] gap-6 items-start">
            <Show when={tab() === "rounds"}>
              <div class="space-y-4 lg:col-span-1">
                <h2 class="flex items-center gap-2 font-bold">
                  <span class="icon-[fluent--calendar-agenda-20-regular] w-5 h-5 text-primary" />
                  {t("tournament.pool.rounds")}
                </h2>
                <div class="border border-primary/25 rounded-lg p-4 space-y-3">
                  <div class="flex items-center gap-2">
                    <span class="icon-[fluent--play-circle-20-regular] w-5 h-5 text-primary" />
                    <strong>{t("tournament.admin.currentRound")}</strong>
                  </div>
                  <Select
                    label={t("tournament.admin.currentRound")}
                    value={[String(tournament()?.current_round_id ?? "")]}
                    disabled={tournament()?.lifecycle !== "running"}
                    items={(rounds() ?? []).map((round) => ({
                      label: `${round.order_index}. ${round.name}`,
                      value: String(round.id),
                    }))}
                    onValueChange={(event) => {
                      const roundId = Number(event.value[0]);
                      if (!roundId) return;
                      const round = rounds()?.find((item) => item.id === roundId);
                      if (!round) return;
                      askConfirm(
                        `${t("tournament.admin.enterRound")}：${round.name}`,
                        async () => {
                          await enterRound(id(), round.id, true);
                          await refetchTournament();
                          await refetchRounds();
                        },
                        t("tournament.admin.confirm")
                      );
                    }}
                  />
                  <Show when={tournament()?.lifecycle !== "running"}>
                    <p class="text-xs opacity-60">{t("tournament.admin.roundOnlyRunning")}</p>
                  </Show>
                </div>
                <div class="border border-layer-content/15 rounded-lg p-4 space-y-3">
                  <div class="flex items-center gap-2">
                    <span class="icon-[fluent--add-circle-20-regular] w-5 h-5 text-primary" />
                    <strong class="text-sm">{t("tournament.pool.addRound")}</strong>
                  </div>
                  <div class="grid sm:grid-cols-[minmax(0,1fr)_5rem_auto] gap-2 items-end">
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
                              release_audience: roundAudience(),
                              release_timing: roundReleaseTiming(),
                              end_mode: roundEndMode(),
                              release_at: roundReleaseAt() ? DateTime.fromISO(roundReleaseAt()).toUTC() : null,
                            }),
                          refetchRounds
                        )
                      }
                    >
                      <span class="icon-[fluent--add-20-regular] w-4 h-4" />
                      {t("general.actions.add.title")}
                    </Button>
                  </div>
                </div>
                <For
                  each={rounds()}
                  fallback={
                    <div class="border border-dashed border-layer-content/20 rounded-lg p-8 text-center opacity-60">
                      <span class="icon-[fluent--calendar-agenda-20-regular] w-8 h-8 mx-auto mb-2 block" />
                      <p>{t("tournament.pool.emptyRounds")}</p>
                    </div>
                  }
                >
                  {(round) => (
                    <div
                      class="p-4 border border-layer-content/10 hover:border-layer-content/20 rounded-lg space-y-3 transition-colors bg-layer-content/[0.02]"
                      classList={{ "border-primary/40 bg-primary/[0.04]": tournament()?.current_round_id === round.id }}
                    >
                      <div class="flex items-center gap-3">
                        <span class="icon-[fluent--calendar-agenda-20-regular] w-5 h-5 text-primary" />
                        <span class="font-mono text-primary text-sm font-bold w-6">{round.order_index}</span>
                        <strong class="flex-1">{round.name}</strong>
                        <Show when={tournament()?.current_round_id === round.id}>
                          <span class="text-xs text-primary font-bold">{t("tournament.admin.currentRound")}</span>
                        </Show>
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
                        <Show
                          when={tournament()?.lifecycle === "running" && tournament()?.current_round_id !== round.id}
                        >
                          <Button
                            size="sm"
                            ghost
                            title={t("tournament.admin.enterRound")}
                            onClick={() =>
                              askConfirm(
                                `${t("tournament.admin.enterRound")}：${round.name}`,
                                async () => {
                                  await enterRound(id(), round.id, true);
                                  await refetchTournament();
                                  await refetchRounds();
                                },
                                t("tournament.admin.confirm")
                              )
                            }
                          >
                            <span class="icon-[fluent--play-20-regular] w-4 h-4" />
                            {t("tournament.admin.enterRound")}
                          </Button>
                        </Show>
                      </div>
                      <div class="border-t border-layer-content/10 pt-3 space-y-2">
                        <div class="flex items-center gap-2 text-xs font-bold opacity-70">
                          <span class="icon-[fluent--tag-20-regular] w-4 h-4" />
                          {t("tournament.pool.tags")}
                          <span class="font-normal opacity-60">
                            {chartTags()?.filter((tag) => tag.round_id === round.id).length ?? 0}
                          </span>
                        </div>
                        <div class="flex flex-wrap gap-2">
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
                      <div class="flex flex-wrap items-center gap-2 text-xs">
                        <span class="inline-flex items-center gap-1 px-2 py-1 rounded bg-layer-content/10 opacity-70">
                          <span class="icon-[fluent--megaphone-loud-20-regular] w-3.5 h-3.5" />
                          {t(
                            `tournament.admin.releaseTimingOptions.${round.release_timing === "on_enter" ? "onEnter" : round.release_timing === "on_end" ? "onEnd" : "immediate"}`
                          )}
                        </span>
                        <Show when={round.manually_released}>
                          <span class="text-primary">{t("tournament.admin.manuallyReleased")}</span>
                        </Show>
                        <Show when={round.ended_at}>
                          <span class="text-success">{t("tournament.admin.roundEnded")}</span>
                        </Show>
                        <Show
                          when={tournament()?.lifecycle === "running" && tournament()?.current_round_id === round.id}
                        >
                          <Button
                            size="sm"
                            ghost
                            onClick={() => run(async () => await endRound(id(), round.id), refetchRounds)}
                          >
                            <span class="icon-[fluent--stop-20-regular] w-4 h-4" />
                            {t("tournament.admin.endRound")}
                          </Button>
                        </Show>
                        <Show when={!round.started_at && !round.ended_at && !round.manually_released}>
                          <Button
                            size="sm"
                            ghost
                            onClick={() => run(async () => await releaseRound(id(), round.id), refetchRounds)}
                          >
                            <span class="icon-[fluent--megaphone-loud-20-regular] w-4 h-4" />
                            {t("tournament.admin.releaseRound")}
                          </Button>
                        </Show>
                        <Show when={round.manually_released && !round.started_at && !round.ended_at}>
                          <Button
                            size="sm"
                            ghost
                            onClick={() => run(async () => await withdrawRoundRelease(id(), round.id), refetchRounds)}
                          >
                            {t("tournament.admin.withdrawRelease")}
                          </Button>
                        </Show>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>
            <Show when={tab() === "rounds"}>
              <aside class="lg:col-span-1 lg:sticky lg:top-4 space-y-4">
                <div class="border border-layer-content/15 rounded-lg p-4 space-y-4">
                  <div>
                    <h2 class="flex items-center gap-2 font-bold">
                      <span class="icon-[fluent--tag-20-regular] w-5 h-5 text-primary" />
                      {t("tournament.pool.tags")}
                    </h2>
                    <p class="text-xs opacity-60 mt-1">{t("tournament.admin.tagPanelDescription")}</p>
                  </div>
                  <div class="border border-primary/20 bg-primary/[0.03] rounded-lg p-3 space-y-3">
                    <strong class="text-sm">{t("tournament.admin.addTag")}</strong>
                    <Select
                      size="sm"
                      label={t("tournament.pool.chooseRound")}
                      placeholder={t("tournament.pool.chooseRound")}
                      value={tagRound() ? [tagRound()] : []}
                      onValueChange={(e) => setTagRound(e.value[0] ?? "")}
                      items={(rounds() ?? []).map((r) => ({ label: r.name, value: String(r.id) }))}
                    />
                    <Input
                      size="sm"
                      title={t("tournament.pool.tagName")}
                      placeholder={t("tournament.pool.tagName")}
                      value={tagName()}
                      onInput={(event) => setTagName(event.currentTarget.value)}
                    />
                    <Button
                      class="w-full"
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
                      <span class="icon-[fluent--add-20-regular] w-4 h-4" />
                      {t("general.actions.add.title")}
                    </Button>
                  </div>
                </div>
                <div class="border border-layer-content/15 rounded-lg p-4 space-y-4">
                  <div class="flex items-center justify-between">
                    <strong class="text-sm">{t("tournament.admin.tagOverview")}</strong>
                    <span class="text-xs opacity-50">{chartTags()?.length ?? 0}</span>
                  </div>
                  <For each={rounds()} fallback={<p class="text-sm opacity-50">{t("tournament.pool.emptyRounds")}</p>}>
                    {(round) => {
                      const items = () => chartTags()?.filter((tag) => tag.round_id === round.id) ?? [];
                      return (
                        <div class="space-y-2">
                          <div class="flex items-center gap-2 text-xs font-bold opacity-70">
                            <span class="icon-[fluent--calendar-agenda-20-regular] w-4 h-4" />
                            {round.name}
                            <span class="opacity-50">{items().length}</span>
                          </div>
                          <div class="flex flex-wrap gap-2 pl-6">
                            <For
                              each={items()}
                              fallback={<span class="text-xs opacity-40">{t("tournament.pool.noTags")}</span>}
                            >
                              {(tag) => (
                                <div class="inline-flex max-w-full items-center gap-1 rounded-md bg-layer-content/10 pl-2 pr-1 py-1 text-xs">
                                  <span class="icon-[fluent--tag-20-regular] w-3.5 h-3.5 shrink-0" />
                                  <span class="truncate">{tag.name}</span>
                                  <button
                                    type="button"
                                    class="shrink-0 p-0.5 opacity-50 hover:opacity-100 hover:text-error"
                                    aria-label={t("general.actions.delete.title")}
                                    onClick={() =>
                                      askConfirm(`删除标签 "${tag.name}"？`, async () => {
                                        await deleteChartTag(id(), tag.id);
                                        await refetchChartTags();
                                      })
                                    }
                                  >
                                    <span class="icon-[fluent--dismiss-12-regular] w-3 h-3" />
                                  </button>
                                </div>
                              )}
                            </For>
                          </div>
                        </div>
                      );
                    }}
                  </For>
                </div>
              </aside>
            </Show>
            <Show when={tab() === "charts"}>
              <div class="space-y-4 lg:col-span-2">
                <div class="flex items-center gap-2">
                  <h2 class="font-bold">{t("tournament.pool.charts")}</h2>
                  <span class="flex-1" />
                  <Button size="sm" ghost onClick={() => setShowLibraryAdd(!showLibraryAdd())}>
                    <span class="icon-[fluent--add-20-regular] w-4 h-4" />
                    {t("general.actions.add.title")}
                    <span
                      class={`${showLibraryAdd() ? "icon-[fluent--chevron-up-20-regular]" : "icon-[fluent--chevron-down-20-regular]"} w-4 h-4`}
                    />
                  </Button>
                </div>
                <Show when={showLibraryAdd()}>
                  <div class="border border-primary/25 rounded-lg p-4 space-y-4 bg-primary/[0.03]">
                    <div class="flex gap-2 flex-wrap">
                      <Select
                        size="sm"
                        class="min-w-40"
                        placeholder={t("tournament.pool.chooseRound")}
                        value={libraryRound() ? [libraryRound()] : []}
                        onValueChange={(e) => {
                          setLibraryRound(e.value[0] ?? "");
                          setLibraryTag("");
                        }}
                        items={(rounds() ?? []).map((r) => ({ label: r.name, value: String(r.id) }))}
                      />
                      <Select
                        size="sm"
                        class="min-w-40"
                        placeholder={t("tournament.pool.chooseTag")}
                        disabled={!libraryRound()}
                        value={libraryTag() ? [libraryTag()] : []}
                        onValueChange={(e) => setLibraryTag(e.value[0] ?? "")}
                        items={(chartTags()?.filter((tag) => tag.round_id === Number(libraryRound())) ?? []).map(
                          (tag) => ({
                            label: tag.name,
                            value: String(tag.id),
                          })
                        )}
                      />
                    </div>
                    <Select
                      size="sm"
                      label={t("tournament.charts.visibility")}
                      value={libraryVisibility() ? [libraryVisibility()!] : []}
                      onValueChange={(e) => setLibraryVisibility(e.value[0] as ChartVisibility)}
                      items={[
                        { label: t("tournament.chartVisibility.public"), value: "public" },
                        { label: t("tournament.chartVisibility.after_archive"), value: "after_archive" },
                        { label: t("tournament.chartVisibility.private"), value: "private" },
                      ]}
                    />
                    <Select
                      size="sm"
                      label={t("tournament.charts.source")}
                      value={[chartImportMethod()]}
                      onValueChange={(e) => {
                        const value = e.value[0] as ChartImportMethod;
                        setChartImportMethod(value);
                        if (value === "custom") {
                          setChartSource("personal");
                          resetPersonalForm();
                        }
                        setMetadataLocked(value !== "custom");
                        setPhiraDialogOpen(value === "phira");
                      }}
                      items={[
                        { label: t("tournament.charts.customImport"), value: "custom" },
                        { label: t("tournament.charts.libraryImport"), value: "library" },
                        { label: t("tournament.charts.phiraImport"), value: "phira" },
                      ]}
                    />
                    <Show when={chartImportMethod() === "custom"}>
                      <Select
                        size="sm"
                        label={t("tournament.charts.source")}
                        value={["personal"]}
                        disabled
                        items={[{ label: t("tournament.charts.personalUpload"), value: "personal" }]}
                      />
                    </Show>
                    <Show when={chartImportMethod() === "library"}>
                      <Select
                        size="sm"
                        label={t("tournament.charts.source")}
                        value={[chartSource()]}
                        onValueChange={(e) => setChartSource(e.value[0] as ChartSourceType)}
                        items={[
                          { label: t("tournament.charts.personalUpload"), value: "personal" },
                          { label: t("tournament.charts.phigrosSource"), value: "phigros" },
                          { label: t("tournament.charts.phiraSource"), value: "phira" },
                        ]}
                      />
                    </Show>
                    <div class="grid sm:grid-cols-[1fr_auto] gap-2">
                      <Select
                        size="sm"
                        placeholder={t("tournament.charts.select")}
                        value={librarySelection() ? [librarySelection()] : []}
                        disabled={chartImportMethod() !== "library"}
                        onValueChange={(e) => selectLibraryChart(e.value[0] ?? "")}
                        items={(chartLibrary() ?? [])
                          .filter((chart) => chart.source_type === chartSource())
                          .map((chart) => ({
                            label: `${chart.title} · ${chart.difficulty}`,
                            value: String(chart.id),
                          }))}
                      />
                      <Input
                        size="sm"
                        type="number"
                        step="0.01"
                        min="0"
                        title={t("tournament.charts.weight")}
                        value={libraryWeight()}
                        onInput={(e) => setLibraryWeight(e.currentTarget.value)}
                      />
                    </div>
                    <Dialog
                      open={phiraDialogOpen()}
                      onOpenChange={(event) => setPhiraDialogOpen(event.open)}
                      btnContent=""
                      class="hidden"
                      size="sm"
                    >
                      <div class="min-w-72 space-y-4">
                        <Input
                          title={t("tournament.charts.phiraId")}
                          type="number"
                          value={phiraId() === "-" ? "" : phiraId()}
                          onInput={(e) => setPhiraId(e.currentTarget.value)}
                        />
                        <Button
                          level="primary"
                          class="w-full"
                          disabled={!phiraId() || phiraId() === "-"}
                          onClick={() =>
                            run(async () => {
                              const chart = await importPhiraChart(Number(phiraId()));
                              setLibrarySelection(String(chart.id));
                              setLibraryTitle(chart.title);
                              setLibraryArtist(chart.artist);
                              setLibraryCharter(chart.charter);
                              setLibraryDifficulty(chart.difficulty);
                              setLibraryLevel(String(chart.level_constant));
                              setLibraryCover(chart.cover ?? "");
                              setChartDescription("");
                              setMetadataLocked(true);
                              setPhiraId("");
                              setPhiraDialogOpen(false);
                              setChartImportMethod("library");
                              setChartSource("phira");
                            })
                          }
                        >
                          {t("general.actions.import.title")}
                        </Button>
                      </div>
                    </Dialog>
                    <div class="border-t border-layer-content/10 pt-4 space-y-2">
                      <div class="text-sm font-bold">{t("tournament.pool.chartTitle")}</div>
                      <div class="grid sm:grid-cols-2 gap-2">
                        <Input
                          size="sm"
                          noLabel
                          placeholder={t("tournament.pool.chartTitle")}
                          value={libraryTitle()}
                          onInput={(e) => setLibraryTitle(e.currentTarget.value)}
                          disabled={metadataLocked()}
                        />
                        <Input
                          size="sm"
                          noLabel
                          placeholder={t("tournament.charts.artist")}
                          value={libraryArtist()}
                          onInput={(e) => setLibraryArtist(e.currentTarget.value)}
                          disabled={metadataLocked()}
                        />
                        <Input
                          size="sm"
                          noLabel
                          placeholder={t("tournament.charts.charter")}
                          value={libraryCharter()}
                          onInput={(e) => setLibraryCharter(e.currentTarget.value)}
                          disabled={metadataLocked()}
                        />
                        <Input
                          size="sm"
                          noLabel
                          placeholder={t("tournament.pool.difficulty")}
                          value={libraryDifficulty()}
                          onInput={(e) => setLibraryDifficulty(e.currentTarget.value)}
                          disabled={metadataLocked()}
                        />
                        <Input
                          size="sm"
                          noLabel
                          type="number"
                          step="0.1"
                          placeholder={t("tournament.charts.constant")}
                          value={libraryLevel()}
                          onInput={(e) => setLibraryLevel(e.currentTarget.value)}
                          disabled={metadataLocked()}
                        />
                      </div>
                      <div class="flex flex-col space-y-1">
                        <span class="label">{t("tournament.charts.description")}</span>
                        <textarea
                          class="input input-md min-h-24 p-3"
                          value={chartDescription()}
                          onInput={(e) => setChartDescription(e.currentTarget.value)}
                          disabled={false}
                        />
                      </div>
                      <label class="btn btn-sm cursor-pointer w-fit">
                        <span class="icon-[fluent--image-add-20-regular] w-4 h-4" />
                        <span class="max-w-32 truncate">{chartCover()?.name || t("tournament.fields.cover")}</span>
                        <input
                          class="hidden"
                          type="file"
                          accept="image/*"
                          onChange={(e) => setChartCover(e.currentTarget.files?.[0])}
                          disabled={metadataLocked()}
                        />
                      </label>
                      <Show when={metadataLocked() && libraryCover()}>
                        <div class="text-xs opacity-60">
                          {t("tournament.charts.cover")}: {libraryCover()}
                        </div>
                      </Show>
                    </div>
                    <Button
                      level="primary"
                      class="w-full"
                      disabled={
                        !libraryRound() ||
                        !libraryTag() ||
                        !libraryVisibility() ||
                        (chartImportMethod() === "library" && !librarySelection()) ||
                        (chartImportMethod() === "custom" && !libraryTitle().trim())
                      }
                      onClick={() =>
                        run(async () => {
                          let cover = libraryCover() || undefined;
                          if (chartImportMethod() === "custom" && chartCover()) {
                            cover = (await uploadMedia(chartCover()!)).hash;
                          }
                          await addTournamentChartLibrary(id(), {
                            chart_library_id:
                              chartImportMethod() === "library" ? Number(librarySelection()) : undefined,
                            round_id: Number(libraryRound()),
                            tag_id: Number(libraryTag()),
                            visibility: libraryVisibility()!,
                            order_index:
                              tournamentChartLibrary()?.filter((item) => item.link.tag_id === Number(libraryTag()))
                                .length ?? 0,
                            weight_millionths: Math.round((Number(libraryWeight()) || 1) * 1_000_000),
                            description: chartDescription().trim() || undefined,
                            title: chartImportMethod() === "custom" ? libraryTitle().trim() : undefined,
                            artist: chartImportMethod() === "custom" ? libraryArtist().trim() : undefined,
                            charter: chartImportMethod() === "custom" ? libraryCharter().trim() : undefined,
                            difficulty: chartImportMethod() === "custom" ? libraryDifficulty().trim() : undefined,
                            level_constant: chartImportMethod() === "custom" ? Number(libraryLevel()) || 0 : undefined,
                            cover: chartImportMethod() === "custom" ? cover : undefined,
                            metadata: {},
                          });
                          resetPersonalForm();
                          setChartCover(undefined);
                          setLibraryVisibility(undefined);
                        }, refetchTournamentChartLibrary)
                      }
                    >
                      {t("general.actions.add.title")}
                    </Button>
                  </div>
                </Show>
                <For each={tournamentChartLibrary()}>
                  {(item) => (
                    <div class="group p-3 border border-primary/20 hover:border-primary/40 rounded-lg transition-colors bg-primary/[0.03]">
                      <div class="flex items-center gap-3 mb-2">
                        <span class="font-mono text-xs text-primary font-bold shrink-0">{item.chart.difficulty}</span>
                        <strong class="flex-1 truncate">{item.chart.title}</strong>
                        <span class="text-xs opacity-50">
                          {chartTags()?.find((tag) => tag.id === item.link.tag_id)?.name}
                        </span>
                        <Button
                          square
                          size="sm"
                          ghost
                          level="error"
                          title={t("general.actions.delete.title")}
                          onClick={() =>
                            run(
                              async () => await removeTournamentChartLibrary(id(), item.link.id),
                              refetchTournamentChartLibrary
                            )
                          }
                        >
                          <span class="icon-[fluent--link-dismiss-20-regular] w-4 h-4" />
                        </Button>
                      </div>
                      <div class="flex gap-2 items-end text-xs">
                        <Input
                          size="sm"
                          title="order"
                          type="number"
                          value={String(item.link.order_index)}
                          onChange={(e) =>
                            run(
                              async () =>
                                await updateTournamentChartLibrary(id(), item.link.id, {
                                  ...item.link,
                                  order_index: Number(e.currentTarget.value),
                                }),
                              refetchTournamentChartLibrary
                            )
                          }
                        />
                        <Input
                          size="sm"
                          title={t("tournament.charts.weight")}
                          type="number"
                          step="0.01"
                          min="0"
                          value={(item.link.weight_millionths / 1_000_000).toFixed(2)}
                          onChange={(e) =>
                            run(
                              async () =>
                                await updateTournamentChartLibrary(id(), item.link.id, {
                                  ...item.link,
                                  weight_millionths: Math.round(Number(e.currentTarget.value) * 1_000_000),
                                }),
                              refetchTournamentChartLibrary
                            )
                          }
                        />
                        <span class="opacity-50 truncate">
                          {item.chart.artist || "--"} · {item.chart.charter || "--"}
                        </span>
                        <span class="font-mono opacity-50">{item.chart.level_constant?.toFixed(1) ?? "0.0"}</span>
                      </div>
                    </div>
                  )}
                </For>
                <div class="border-t border-layer-content/15 pt-4 space-y-3">
                  <h3 class="text-sm font-bold">{t("tournament.pool.charts")}</h3>
                  <For each={charts()}>
                    {(chart) => (
                      <div class="group p-3 border border-layer-content/10 hover:border-layer-content/20 rounded-lg transition-colors bg-layer-content/[0.02]">
                        <div class="flex items-center gap-3 mb-1.5">
                          <span class="font-mono text-xs text-primary font-bold shrink-0 min-w-10">
                            {chart.difficulty}
                          </span>
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
              </div>
            </Show>
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
          <p class="font-bold">{confirmActionLabel()}</p>
          <p class="text-sm">{confirmMsg()}</p>
          <div class="flex justify-end gap-2">
            <Button size="sm" ghost onClick={() => setConfirmOpen(false)}>
              {t("general.actions.cancel.title")}
            </Button>
            <Button
              size="sm"
              level="primary"
              onClick={() =>
                run(async () => {
                  await confirmAction()();
                  setConfirmOpen(false);
                })
              }
            >
              {confirmActionLabel()}
            </Button>
          </div>
        </div>
      </Dialog>
    </main>
  );
}
