import { getCharts, getChartTags, getRounds } from "@api/tournament";
import SidebarLayout from "@blocks/sidebar-layout";
import { mediaPath } from "@lib/utils/media";
import { createBreakpoints } from "@solid-primitives/media";
import { useParams, useSearchParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { breakpoints, fullTheme, t } from "@storage/theme";
import Button from "@widgets/button";
import Input from "@widgets/input";
import LoadingTips from "@widgets/loading-tips";
import Picture from "@widgets/picture";
import type { TreeNode } from "@widgets/treeview";
import TreeView from "@widgets/treeview";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { createMemo, createResource, createSignal, Match, Show, Switch } from "solid-js";
import { Transition } from "solid-transition-group";

export default function () {
  const params = useParams();
  const tournamentId = () => Number(params.tournament);
  const [searchParams] = useSearchParams();
  const [rounds] = createResource(tournamentId, getRounds);
  const [tags] = createResource(tournamentId, getChartTags);
  const [charts] = createResource(tournamentId, getCharts);
  const [search, setSearch] = createSignal("");
  const [showLeftSidebar, setShowLeftSidebar] = createSignal(false);
  const matches = createBreakpoints(breakpoints);

  const selectedChartId = createMemo(() => Number.parseInt(String(searchParams.chart ?? ""), 10) || null);
  const selectedChart = createMemo(() => charts()?.find((chart) => chart.id === selectedChartId()));
  const selectedRound = createMemo(() => rounds()?.find((round) => round.id === selectedChart()?.round_id));
  const selectedTag = createMemo(() => tags()?.find((tag) => tag.id === selectedChart()?.tag_id));
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
          children: (charts() ?? [])
            .filter((chart) => chart.tag_id === tag.id)
            .filter((chart) =>
              [chart.title, chart.artist, chart.charter, chart.difficulty, tag.name, round.name]
                .join(" ")
                .toLowerCase()
                .includes(query)
            )
            .map((chart) => ({
              id: chart.id,
              name: chart.title,
              type: "item" as const,
              searchValue: String(chart.id),
              link: `/tournaments/${tournamentId()}/charts?chart=${chart.id}`,
              icon: "icon-[fluent--music-note-2-20-regular]",
              extraPart: <span class="text-xs opacity-50">{chart.difficulty}</span>,
              children: [],
            })),
        })),
    }));
  });

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
              <Match when={rounds.loading || tags.loading || charts.loading}>
                <div class="p-4 flex justify-center">
                  <LoadingTips />
                </div>
              </Match>
              <Match when={tree().length > 0}>
                <TreeView
                  tree={tree()}
                  activeSearchParams="chart"
                  highlightPaths={
                    selectedChart()
                      ? [
                          `round-${selectedChart()!.round_id}`,
                          `tag-${selectedChart()!.tag_id}`,
                          String(selectedChart()!.id),
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
                  <Picture
                    class="aspect-square rounded-lg border border-layer-content/15"
                    src={mediaPath(chart().cover)}
                    alt={chart().title}
                  />
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
                        <dd class="font-mono font-bold mt-1">{(chart().weight_millionths / 1_000_000).toFixed(2)}x</dd>
                      </div>
                    </dl>
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
