import { handleHttpError, toastSuccess } from "@api";
import { createLibraryChart, deleteLibraryChart, getChartLibrary, getTournaments } from "@api/tournament";
import { mediaPath } from "@lib/utils/media";
import type { ChartLibraryStatus } from "@models/tournament";
import { Permission } from "@models/user";
import { useNavigate, useSearchParams } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Dialog from "@widgets/dialog";
import Input from "@widgets/input";
import LoadingTips from "@widgets/loading-tips";
import Picture from "@widgets/picture";
import Select from "@widgets/select";
import { createMemo, createResource, createSignal, For, Match, Show, Switch } from "solid-js";

const EMPTY_FORM = {
  title: "",
  artist: "",
  charter: "",
  difficulty: "",
  level_constant: "0",
  cover: "",
};

const statusBadgeClass = (status?: ChartLibraryStatus) => {
  switch (status) {
    case "pending":
      return "bg-warning/80 text-white";
    case "rejected":
      return "bg-error/80 text-white";
    default:
      return "bg-success/80 text-white";
  }
};

const statusLabel = (status?: ChartLibraryStatus) => {
  switch (status) {
    case "pending":
      return t("tournament.charts.pending");
    case "rejected":
      return t("tournament.charts.rejected");
    default:
      return t("tournament.charts.approved");
  }
};

export default function () {
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const [chartLibrary, { refetch: refetchLibrary }] = createResource(getChartLibrary);
  const [tournaments] = createResource(getTournaments);
  const [search, setSearch] = createSignal("");
  const [tournamentFilter, setTournamentFilter] = createSignal("");
  const [sourceFilter, setSourceFilter] = createSignal("");
  const [difficultyFilter, setDifficultyFilter] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [uploadOpen, setUploadOpen] = createSignal(false);
  const [form, setForm] = createSignal({ ...EMPTY_FORM });
  const [uploadSource, setUploadSource] = createSignal<"personal" | "phigros" | "phira">("personal");
  const selectedId = createMemo(() => Number(searchParams.chart) || null);

  const canManage = createMemo(
    () =>
      accountStore.permissions.includes(Permission.ChartLibrary) || accountStore.permissions.includes(Permission.DevOps)
  );
  const canModify = (chart: { created_by?: number }) =>
    accountStore.permissions.includes(Permission.DevOps) ||
    (accountStore.permissions.includes(Permission.ChartLibrary) && chart.created_by === accountStore.id);

  const filterOptions = createMemo(() => {
    const charts = chartLibrary() ?? [];
    const archivedTournaments = (tournaments() ?? [])
      .filter((tournament) => tournament.lifecycle === "archived")
      .sort((left, right) => left.name.localeCompare(right.name));
    const sources = [
      { label: t("tournament.charts.localSource"), value: "personal" },
      { label: t("tournament.charts.phigrosSource"), value: "phigros" },
      { label: t("tournament.charts.phiraSource"), value: "phira" },
    ];
    const difficulties = [...new Set(charts.map((chart) => chart.difficulty).filter(Boolean))].sort();
    return { archivedTournaments, sources, difficulties };
  });
  const filteredCharts = createMemo(() => {
    const query = search().trim().toLowerCase();
    return (chartLibrary() ?? []).filter(
      (chart) =>
        [chart.title, chart.artist, chart.charter, chart.difficulty].join(" ").toLowerCase().includes(query) &&
        (!tournamentFilter() || (chart.tournaments ?? "").split(", ").includes(tournamentFilter())) &&
        (!sourceFilter() || chart.source_type === sourceFilter()) &&
        (!difficultyFilter() || chart.difficulty === difficultyFilter())
    );
  });
  const selectedChart = createMemo(() => chartLibrary()?.find((chart) => chart.id === selectedId()));
  const sourceLabel = (sourceType?: string) => {
    switch (sourceType) {
      case "personal":
        return t("tournament.charts.localSource");
      case "phira":
        return t("tournament.charts.phiraSource");
      case "phigros":
        return t("tournament.charts.phigrosSource");
      default:
        return sourceType || t("tournament.charts.source");
    }
  };

  const updateForm = (field: keyof typeof EMPTY_FORM, value: string) =>
    setForm((prev) => ({ ...prev, [field]: value }));

  const resetForm = () => {
    setForm({ ...EMPTY_FORM });
    setUploadSource("personal");
  };

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

  const submitUpload = () =>
    run(async () => {
      const level = Number(form().level_constant);
      if (!form().title.trim() || !form().difficulty.trim() || Number.isNaN(level)) return;
      await createLibraryChart({
        source_type: uploadSource(),
        title: form().title.trim(),
        artist: form().artist.trim(),
        charter: form().charter.trim(),
        difficulty: form().difficulty.trim(),
        level_constant: level,
        cover: form().cover.trim() || undefined,
        metadata: {},
      });
      toastSuccess(t("tournament.charts.uploadSuccess"));
      resetForm();
      setUploadOpen(false);
    }, refetchLibrary);

  const removeChart = (id: number) =>
    run(async () => {
      await deleteLibraryChart(id);
      toastSuccess(t("tournament.charts.deleteSuccess"));
      setSearchParams({ chart: undefined });
    }, refetchLibrary);

  return (
    <>
      <Title page={t("tournament.charts.library")} route="/charts" />
      <div class="w-full max-w-7xl mx-auto p-4 lg:p-8">
        <div class="flex flex-col gap-6">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div>
              <h1 class="text-3xl font-bold">{t("tournament.charts.library")}</h1>
              <p class="opacity-60 mt-2">{t("tournament.pool.empty")}</p>
            </div>
            <Show when={canManage()}>
              <div class="flex gap-2">
                <Button level="info" size="sm" onClick={() => navigate("/charts/review")}>
                  <span class="icon-[fluent--clipboard-check-20-regular] w-5 h-5 mr-1" />
                  {t("tournament.charts.review")}
                </Button>
                <Button
                  level="primary"
                  size="sm"
                  onClick={() => {
                    resetForm();
                    setUploadOpen(true);
                  }}
                >
                  <span class="icon-[fluent--arrow-upload-20-regular] w-5 h-5 mr-1" />
                  {t("tournament.charts.upload")}
                </Button>
              </div>
            </Show>
          </div>
          <Input
            size="sm"
            icon={<span class="icon-[fluent--search-20-regular] w-5 h-5" />}
            placeholder={t("tournament.charts.search")}
            value={search()}
            onInput={(event) => setSearch(event.currentTarget.value)}
          />
          <div class="grid gap-3 sm:grid-cols-3">
            <Select
              size="sm"
              placeholder={t("tournament.charts.allTournaments")}
              value={tournamentFilter() ? [tournamentFilter()] : []}
              onValueChange={(event) => setTournamentFilter(event.value[0] ?? "")}
              items={[
                { label: t("tournament.charts.allTournaments"), value: "" },
                ...filterOptions().archivedTournaments.map((tournament) => ({
                  label: tournament.name,
                  value: tournament.name,
                })),
              ]}
            />
            <Select
              size="sm"
              placeholder={t("tournament.charts.allSources")}
              value={sourceFilter() ? [sourceFilter()] : []}
              onValueChange={(event) => setSourceFilter(event.value[0] ?? "")}
              items={[{ label: t("tournament.charts.allSources"), value: "" }, ...filterOptions().sources]}
            />
            <Select
              size="sm"
              placeholder={t("tournament.charts.allDifficulties")}
              value={difficultyFilter() ? [difficultyFilter()] : []}
              onValueChange={(event) => setDifficultyFilter(event.value[0] ?? "")}
              items={[
                { label: t("tournament.charts.allDifficulties"), value: "" },
                ...filterOptions().difficulties.map((value) => ({ label: value, value: value! })),
              ]}
            />
          </div>
          <Switch>
            <Match when={chartLibrary.loading}>
              <div class="min-h-48 flex items-center justify-center">
                <LoadingTips />
              </div>
            </Match>
            <Match when={filteredCharts().length > 0}>
              <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                <For each={filteredCharts()}>
                  {(chart) => (
                    <button
                      type="button"
                      class="text-left border border-layer-content/15 rounded-lg overflow-hidden hover:border-primary transition-colors"
                      classList={{ "border-primary": selectedId() === chart.id }}
                      onClick={() => setSearchParams({ chart: chart.id })}
                    >
                      <div class="relative">
                        <Picture class="aspect-video w-full" src={mediaPath(chart.cover)} alt={chart.title} />
                        <span class="absolute top-2 right-2 rounded bg-black/65 px-2 py-1 text-xs text-white">
                          {sourceLabel(chart.source_type)}
                        </span>
                        <Show when={chart.status && chart.status !== "approved"}>
                          <span
                            class={`absolute top-2 left-2 rounded px-2 py-1 text-xs ${statusBadgeClass(chart.status)}`}
                          >
                            {statusLabel(chart.status)}
                          </span>
                        </Show>
                      </div>
                      <div class="p-4">
                        <h2 class="font-bold truncate">{chart.title}</h2>
                        <p class="text-sm opacity-60 truncate mt-1">{chart.artist || "--"}</p>
                        <div class="flex justify-between gap-2 text-sm mt-3">
                          <span>{chart.difficulty}</span>
                          <span class="font-mono">{chart.level_constant.toFixed(1)}</span>
                        </div>
                      </div>
                    </button>
                  )}
                </For>
              </div>
            </Match>
            <Match when={true}>
              <div class="min-h-48 flex items-center justify-center opacity-60 text-center">
                <p>{t("tournament.pool.empty")}</p>
              </div>
            </Match>
          </Switch>
          <Show when={selectedChart()}>
            {(chart) => (
              <section class="border-t border-layer-content/15 pt-6">
                <div class="flex flex-wrap items-start justify-between gap-3">
                  <h2 class="text-xl font-bold">{chart().title}</h2>
                  <Show when={canModify(chart())}>
                    <Button
                      level="error"
                      size="sm"
                      ghost
                      loading={busy()}
                      onClick={() => {
                        if (window.confirm(t("tournament.charts.confirmDelete"))) removeChart(chart().id);
                      }}
                    >
                      <span class="icon-[fluent--delete-20-regular] w-5 h-5 mr-1" />
                      {t("tournament.charts.delete")}
                    </Button>
                  </Show>
                </div>
                <div class="grid sm:grid-cols-3 gap-4 mt-4 text-sm">
                  <div>
                    <span class="opacity-60">{t("tournament.charts.artist")}</span>
                    <p>{chart().artist || "--"}</p>
                  </div>
                  <div>
                    <span class="opacity-60">{t("tournament.charts.charter")}</span>
                    <p>{chart().charter || "--"}</p>
                  </div>
                  <div>
                    <span class="opacity-60">{t("tournament.pool.difficulty")}</span>
                    <p>{chart().difficulty}</p>
                  </div>
                </div>
              </section>
            )}
          </Show>
        </div>
      </div>
      <Dialog open={uploadOpen()} onOpenChange={(event) => setUploadOpen(event.open)} btnContent="" class="hidden">
        <div class="min-w-80 space-y-4">
          <h2 class="text-lg font-bold">{t("tournament.charts.uploadChart")}</h2>
          <Select
            size="sm"
            label={t("tournament.charts.source")}
            value={[uploadSource()]}
            onValueChange={(event) => setUploadSource(event.value[0] as "personal" | "phigros" | "phira")}
            items={[
              { label: t("tournament.charts.personalUpload"), value: "personal" },
              { label: t("tournament.charts.phigrosSource"), value: "phigros" },
              { label: t("tournament.charts.phiraSource"), value: "phira" },
            ]}
          />
          <Input
            size="sm"
            label={t("tournament.pool.chartTitle")}
            placeholder={t("tournament.pool.chartTitle")}
            value={form().title}
            onInput={(event) => updateForm("title", event.currentTarget.value)}
          />
          <div class="grid sm:grid-cols-2 gap-2">
            <Input
              size="sm"
              label={t("tournament.charts.artist")}
              placeholder={t("tournament.charts.artist")}
              value={form().artist}
              onInput={(event) => updateForm("artist", event.currentTarget.value)}
            />
            <Input
              size="sm"
              label={t("tournament.charts.charter")}
              placeholder={t("tournament.charts.charter")}
              value={form().charter}
              onInput={(event) => updateForm("charter", event.currentTarget.value)}
            />
          </div>
          <div class="grid sm:grid-cols-2 gap-2">
            <Input
              size="sm"
              label={t("tournament.pool.difficulty")}
              placeholder={t("tournament.pool.difficulty")}
              value={form().difficulty}
              onInput={(event) => updateForm("difficulty", event.currentTarget.value)}
            />
            <Input
              size="sm"
              type="number"
              step="0.01"
              min="0"
              label={t("tournament.charts.constant")}
              placeholder={t("tournament.charts.constant")}
              value={form().level_constant}
              onInput={(event) => updateForm("level_constant", event.currentTarget.value)}
            />
          </div>
          <Input
            size="sm"
            label={t("tournament.charts.cover")}
            placeholder={t("tournament.charts.cover")}
            value={form().cover}
            onInput={(event) => updateForm("cover", event.currentTarget.value)}
          />
          <Button
            level="primary"
            class="w-full"
            loading={busy()}
            disabled={!form().title.trim()}
            onClick={submitUpload}
          >
            {t("tournament.charts.upload")}
          </Button>
        </div>
      </Dialog>
    </>
  );
}
