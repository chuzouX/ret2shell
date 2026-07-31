import { handleHttpError, toastSuccess } from "@api";
import { deleteLibraryChart, getPendingCharts, reviewLibraryChart } from "@api/tournament";
import { mediaPath } from "@lib/utils/media";
import type { ChartLibrary } from "@models/tournament";
import { Permission } from "@models/user";
import { useNavigate } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import LoadingTips from "@widgets/loading-tips";
import Picture from "@widgets/picture";
import { createResource, createSignal, For, Match, Switch } from "solid-js";

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

export default function () {
  const navigate = useNavigate();
  const [pendingCharts, { refetch }] = createResource(getPendingCharts);
  const [busy, setBusy] = createSignal(false);

  const canManage = () =>
    accountStore.permissions.includes(Permission.ChartLibrary) || accountStore.permissions.includes(Permission.DevOps);

  if (!canManage()) {
    navigate("/charts");
    return null;
  }

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

  const reviewChart = (chart: ChartLibrary, status: "approved" | "rejected") =>
    run(async () => {
      await reviewLibraryChart(chart.id, status);
      toastSuccess(t("tournament.charts.reviewSuccess"));
    }, refetch);

  const removeChart = (id: number) =>
    run(async () => {
      await deleteLibraryChart(id);
      toastSuccess(t("tournament.charts.deleteSuccess"));
    }, refetch);

  return (
    <>
      <Title page={t("tournament.charts.reviewQueue")} route="/charts/review" />
      <div class="w-full max-w-7xl mx-auto p-4 lg:p-8">
        <div class="flex flex-col gap-6">
          <div class="flex items-center justify-between gap-3">
            <div>
              <h1 class="text-3xl font-bold">{t("tournament.charts.reviewQueue")}</h1>
              <p class="opacity-60 mt-2">{t("tournament.charts.pending")}</p>
            </div>
            <Button level="ghost" size="sm" onClick={() => navigate("/charts")}>
              <span class="icon-[fluent--arrow-left-20-regular] w-5 h-5 mr-1" />
              {t("tournament.charts.library")}
            </Button>
          </div>
          <Switch>
            <Match when={pendingCharts.loading}>
              <div class="min-h-48 flex items-center justify-center">
                <LoadingTips />
              </div>
            </Match>
            <Match when={(pendingCharts() ?? []).length > 0}>
              <div class="grid gap-3 lg:grid-cols-2">
                <For each={pendingCharts()}>
                  {(chart) => (
                    <div class="border border-layer-content/15 rounded-lg overflow-hidden flex flex-col sm:flex-row">
                      <Picture
                        class="aspect-video sm:w-48 sm:aspect-auto object-cover"
                        src={mediaPath(chart.cover)}
                        alt={chart.title}
                      />
                      <div class="p-4 flex-1 flex flex-col gap-2">
                        <div class="flex items-start justify-between gap-2">
                          <h2 class="font-bold truncate">{chart.title}</h2>
                          <span class="rounded bg-warning/80 px-2 py-0.5 text-xs text-white shrink-0">
                            {sourceLabel(chart.source_type)}
                          </span>
                        </div>
                        <p class="text-sm opacity-60 truncate">{chart.artist || "--"}</p>
                        <div class="flex flex-wrap gap-3 text-sm">
                          <span class="opacity-60">
                            {t("tournament.charts.charter")}: {chart.charter || "--"}
                          </span>
                          <span class="opacity-60">
                            {t("tournament.pool.difficulty")}: {chart.difficulty}
                          </span>
                          <span class="font-mono opacity-60">{chart.level_constant.toFixed(1)}</span>
                        </div>
                        <div class="flex gap-2 mt-auto pt-2">
                          <Button
                            level="success"
                            size="sm"
                            loading={busy()}
                            onClick={() => reviewChart(chart, "approved")}
                          >
                            <span class="icon-[fluent--checkmark-20-regular] w-5 h-5 mr-1" />
                            {t("tournament.charts.approve")}
                          </Button>
                          <Button
                            level="error"
                            size="sm"
                            ghost
                            loading={busy()}
                            onClick={() => reviewChart(chart, "rejected")}
                          >
                            <span class="icon-[fluent--dismiss-20-regular] w-5 h-5 mr-1" />
                            {t("tournament.charts.reject")}
                          </Button>
                          <Button
                            level="error"
                            size="sm"
                            ghost
                            loading={busy()}
                            onClick={() => {
                              if (window.confirm(t("tournament.charts.confirmDelete"))) removeChart(chart.id);
                            }}
                          >
                            <span class="icon-[fluent--delete-20-regular] w-5 h-5" />
                          </Button>
                        </div>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Match>
            <Match when={true}>
              <div class="min-h-48 flex items-center justify-center opacity-60 text-center">
                <p>{t("tournament.charts.noPendingCharts")}</p>
              </div>
            </Match>
          </Switch>
        </div>
      </div>
    </>
  );
}
