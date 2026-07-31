import { getChartTags, getRounds, getTournamentChartLibrary } from "@api/tournament";
import SidebarLayout from "@blocks/sidebar-layout";
import { mediaPath } from "@lib/utils/media";
import { createBreakpoints } from "@solid-primitives/media";
import { useParams, useSearchParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { breakpoints, fullTheme, t } from "@storage/theme";
import Article from "@widgets/article";
import Button from "@widgets/button";
import Input from "@widgets/input";
import LoadingTips from "@widgets/loading-tips";
import Picture from "@widgets/picture";
import type { TreeNode } from "@widgets/treeview";
import TreeView from "@widgets/treeview";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { createEffect, createMemo, createResource, createSignal, For, Match, Show, Switch } from "solid-js";
import { Transition } from "solid-transition-group";

export default function () {
  const params = useParams();
  const tournamentId = () => Number(params.tournament);
  const [searchParams, setSearchParams] = useSearchParams();
  const [rounds] = createResource(tournamentId, getRounds);
  const [tags] = createResource(tournamentId, getChartTags);
  const [chartLibrary] = createResource(tournamentId, getTournamentChartLibrary);
  const [search, setSearch] = createSignal("");
  const [openCharts, setOpenCharts] = createSignal<number[]>([]);
  const [showLeftSidebar, setShowLeftSidebar] = createSignal(false);
  const matches = createBreakpoints(breakpoints);

  const selectedChartId = createMemo(() => Number.parseInt(String(searchParams.chart ?? ""), 10) || null);
  const selectedItem = createMemo(() => chartLibrary()?.find((item) => item.link.id === selectedChartId()));
  const selectedChart = createMemo(() => selectedItem()?.chart);
  const selectedDescription = createMemo(() => selectedItem()?.link.description);
  const selectedRound = createMemo(() => rounds()?.find((round) => round.id === selectedItem()?.link.round_id));
  const selectedTag = createMemo(() => tags()?.find((tag) => tag.id === selectedItem()?.link.tag_id));
  const openChartItems = createMemo(() =>
    openCharts()
      .map((id) => chartLibrary()?.find((item) => item.link.id === id))
      .filter((item): item is NonNullable<typeof item> => Boolean(item))
  );
  createEffect(() => {
    const selected = selectedChartId();
    if (selected && chartLibrary()?.some((item) => item.link.id === selected)) {
      setOpenCharts((items) => (items.includes(selected) ? items : [...items, selected]));
    }
  });
  const sourceLabel = (sourceType?: string, custom = false) => {
    if (custom) return t("tournament.charts.customSource");
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
  const tree = createMemo<TreeNode[]>(() => {
    const query = search().trim().toLowerCase();
    return (rounds() ?? []).map((round) => ({
      id: `round-${round.id}`,
      name: round.name,
      type: "category" as const,
      icon: "icon-[fluent--calendar-agenda-20-regular]",
      children: (tags() ?? [])
        .filter((tag) => tag.round_id === round.id)
        .map((tag) => ({
          id: `tag-${tag.id}`,
          name: tag.name,
          type: "category" as const,
          icon: "icon-[fluent--tag-20-regular]",
          children: (chartLibrary() ?? [])
            .filter((item) => item.link.tag_id === tag.id)
            .filter((chart) =>
              [chart.chart.title, chart.chart.artist, chart.chart.charter, chart.chart.difficulty, tag.name, round.name]
                .join(" ")
                .toLowerCase()
                .includes(query)
            )
            .map((item) => ({
              id: item.link.id,
              name: item.chart.title,
              type: "item" as const,
              searchValue: String(item.link.id),
              link: `/tournaments/${tournamentId()}/charts?chart=${item.link.id}`,
              icon: "icon-[fluent--music-note-2-20-regular]",
              extraPart: <span class="text-xs opacity-50">{item.chart.difficulty}</span>,
              children: [],
            })),
        })),
    }));
  });

  const openChart = (id: number) => {
    setOpenCharts((items) => (items.includes(id) ? items : [...items, id]));
  };

  const closeChart = (id: number) => {
    const remaining = openCharts().filter((item) => item !== id);
    setOpenCharts(remaining);
    if (selectedChartId() === id) {
      const next = remaining.at(-1);
      setSearchParams({ chart: next ? String(next) : undefined });
    }
  };

  const leftBar = () => (
    <div class="h-full flex flex-col">
      <div class="border-b border-b-layer-content/10 px-2 h-16 flex items-center">
        <div class="w-full flex items-center gap-2 px-3">
          <span class="icon-[fluent--library-20-filled] w-5 h-5 text-primary" />
          <strong>{t("tournament.charts.library")}</strong>
        </div>
      </div>
      <div class="flex-1 overflow-hidden">
        <OverlayScrollbarsComponent
          options={{ scrollbars: { theme: `os-theme-${fullTheme()}`, autoHide: "scroll" } }}
          class="relative w-full h-full"
          defer
        >
          <div class="flex flex-col gap-3 p-3 lg:p-5">
            <Input
              class="bg-layer"
              size="sm"
              icon={<span class="icon-[fluent--filter-20-regular] w-5 h-5" />}
              placeholder={t("tournament.charts.search")}
              value={search()}
              onInput={(event) => setSearch(event.currentTarget.value)}
            />
            <Switch>
              <Match when={rounds.loading || tags.loading || chartLibrary.loading}>
                <div class="p-4 flex justify-center">
                  <LoadingTips />
                </div>
              </Match>
              <Match when={tree().length > 0}>
                <TreeView
                  tree={tree()}
                  onNodeClick={(node) => {
                    if (node.type === "item") openChart(Number(node.id));
                  }}
                  activeSearchParams="chart"
                  highlightPaths={
                    selectedItem()
                      ? [
                          `round-${selectedItem()!.link.round_id}`,
                          `tag-${selectedItem()!.link.tag_id}`,
                          String(selectedItem()!.link.id),
                        ]
                      : undefined
                  }
                />
              </Match>
              <Match when={true}>
                <p class="p-4 opacity-60 text-center">{t("tournament.pool.empty")}</p>
              </Match>
            </Switch>
          </div>
        </OverlayScrollbarsComponent>
      </div>
    </div>
  );

  return (
    <>
      <Title page={t("tournament.charts.library")} route={`/tournaments/${tournamentId()}/charts`} />
      <SidebarLayout showLeftBar={showLeftSidebar()} leftBar={leftBar}>
        <div class="flex-1 min-w-0 flex flex-col">
          <Show when={openChartItems().length > 0}>
            <div class="h-12 shrink-0 border-b border-layer-content/10 flex items-stretch overflow-x-auto px-2 gap-1">
              <For each={openChartItems()}>
                {(item) => (
                  <div
                    class="group min-w-36 max-w-56 flex items-center gap-2 px-3 text-xs border-x border-transparent hover:bg-layer-content/5"
                    classList={{ "bg-layer-content/10 border-layer-content/10": item.link.id === selectedChartId() }}
                  >
                    <button
                      type="button"
                      class="min-w-0 flex-1 truncate"
                      onClick={() => {
                        openChart(item.link.id);
                        setSearchParams({ chart: String(item.link.id) });
                      }}
                    >
                      <span class="icon-[fluent--music-note-2-20-regular] w-3.5 h-3.5 mr-1 align-text-bottom opacity-60" />
                      {item.chart.title}
                    </button>
                    <button
                      type="button"
                      class="shrink-0 p-1 rounded opacity-50 hover:opacity-100 hover:bg-layer-content/10"
                      aria-label={t("general.actions.close.title")}
                      onClick={() => closeChart(item.link.id)}
                    >
                      <span class="icon-[fluent--dismiss-12-regular] w-3.5 h-3.5" />
                    </button>
                  </div>
                )}
              </For>
            </div>
          </Show>
          <Show
            when={selectedChart()}
            fallback={
              <div class="flex-1 min-h-[28rem] flex flex-col items-center justify-center gap-5 opacity-50 p-8 text-center">
                <span class="icon-[fluent--music-note-2-20-regular] w-20 h-20" />
                <div>
                  <h1 class="text-2xl font-bold">{t("tournament.charts.select")}</h1>
                  <p class="mt-2">{t("tournament.charts.selectHint")}</p>
                </div>
              </div>
            }
          >
            {(chart) => (
              <article class="w-full max-w-6xl mx-auto p-4 lg:p-8">
                <div class="grid lg:grid-cols-[minmax(18rem,28rem)_1fr] gap-8 items-start">
                  <div class="relative">
                    <Picture
                      class="aspect-square rounded-lg border border-layer-content/15"
                      src={mediaPath(chart().cover)}
                      alt={chart().title}
                    />
                    <span class="absolute top-2 right-2 rounded bg-black/65 px-2 py-1 text-xs text-white">
                      {sourceLabel(chart().source_type, selectedItem()?.link.chart_library_id === null)}
                    </span>
                  </div>
                  <div class="min-w-0 py-2">
                    <div class="flex flex-wrap items-center gap-2 text-sm opacity-60">
                      <span>{selectedRound()?.name}</span>
                      <span>/</span>
                      <span class="inline-flex items-center gap-1">
                        <span class="icon-[fluent--tag-20-regular] w-4 h-4" />
                        {selectedTag()?.name}
                      </span>
                    </div>
                    <h1 class="text-3xl font-bold mt-4 break-words">{chart().title}</h1>
                    <p class="text-lg opacity-60 mt-2">{chart().artist || "--"}</p>
                    <dl class="grid sm:grid-cols-2 gap-x-8 gap-y-5 mt-8 border-y border-layer-content/15 py-6">
                      <div>
                        <dt class="text-xs opacity-50">{t("tournament.charts.charter")}</dt>
                        <dd class="font-bold mt-1">{chart().charter || "--"}</dd>
                      </div>
                      <div>
                        <dt class="text-xs opacity-50">{t("tournament.pool.difficulty")}</dt>
                        <dd class="font-bold mt-1">{chart().difficulty}</dd>
                      </div>
                      <div>
                        <dt class="text-xs opacity-50">{t("tournament.charts.constant")}</dt>
                        <dd class="font-mono font-bold mt-1">{chart().level_constant.toFixed(1)}</dd>
                      </div>
                      <div>
                        <dt class="text-xs opacity-50">{t("tournament.charts.weight")}</dt>
                        <dd class="font-mono font-bold mt-1">
                          {((chart().weight_millionths ?? 1_000_000) / 1_000_000).toFixed(2)}x
                        </dd>
                      </div>
                    </dl>
                    <Show when={selectedDescription()}>
                      <div class="mt-8 border-t border-layer-content/15 pt-6">
                        <Article content={selectedDescription()!} compact />
                      </div>
                    </Show>
                  </div>
                </div>
              </article>
            )}
          </Show>
        </div>
      </SidebarLayout>
      <Transition name="slide-fade-right">
        <Show when={!matches.lg}>
          <Button
            class="fixed bottom-3 right-3 z-30"
            square
            onClick={() => setShowLeftSidebar((value) => !value)}
            type="button"
            title={t("tournament.charts.library")}
          >
            <span
              class={
                showLeftSidebar()
                  ? "icon-[fluent--dismiss-20-regular] w-5 h-5"
                  : "icon-[fluent--library-20-regular] w-5 h-5"
              }
            />
          </Button>
        </Show>
      </Transition>
    </>
  );
}
