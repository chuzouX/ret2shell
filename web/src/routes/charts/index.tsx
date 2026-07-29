import { getChartLibrary, getTournaments } from "@api/tournament";
import { mediaPath } from "@lib/utils/media";
import { useSearchParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Input from "@widgets/input";
import LoadingTips from "@widgets/loading-tips";
import Picture from "@widgets/picture";
import Select from "@widgets/select";
import { createMemo, createResource, createSignal, For, Match, Show, Switch } from "solid-js";

export default function () {
  const [searchParams, setSearchParams] = useSearchParams();
  const [chartLibrary] = createResource(getChartLibrary);
  const [tournaments] = createResource(getTournaments);
  const [search, setSearch] = createSignal("");
  const [tournamentFilter, setTournamentFilter] = createSignal("");
  const [sourceFilter, setSourceFilter] = createSignal("");
  const [difficultyFilter, setDifficultyFilter] = createSignal("");
  const selectedId = createMemo(() => Number(searchParams.chart) || null);
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

  return (
    <>
      <Title page={t("tournament.charts.library")} route="/charts" />
      <div class="w-full max-w-7xl mx-auto p-4 lg:p-8">
        <div class="flex flex-col gap-6">
          <div>
            <h1 class="text-3xl font-bold">{t("tournament.charts.library")}</h1>
            <p class="opacity-60 mt-2">{t("tournament.pool.empty")}</p>
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
                <h2 class="text-xl font-bold">{chart().title}</h2>
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
    </>
  );
}
