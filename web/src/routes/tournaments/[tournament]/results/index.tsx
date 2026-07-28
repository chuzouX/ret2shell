import { handleHttpError } from "@api";
import { uploadMedia } from "@api/media";
import { getCharts, getResults, submitResult } from "@api/tournament";
import { useParams } from "@solidjs/router";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Input from "@widgets/input";
import Select from "@widgets/select";
import { DateTime } from "luxon";
import { createResource, createSignal, For } from "solid-js";

export default function () {
  const params = useParams();
  const id = () => Number(params.tournament);
  const [charts] = createResource(id, getCharts);
  const [results, { refetch }] = createResource(id, getResults);
  const [chartId, setChartId] = createSignal<number>();
  const [score, setScore] = createSignal("");
  const [accuracy, setAccuracy] = createSignal("");
  const [combo, setCombo] = createSignal("");
  const [fullCombo, setFullCombo] = createSignal(false);
  const [allPerfect, setAllPerfect] = createSignal(false);
  const [evidence, setEvidence] = createSignal<string>();
  const [uploading, setUploading] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);

  const upload = async (file?: File) => {
    if (!file) return;
    setUploading(true);
    try {
      setEvidence((await uploadMedia(file)).hash);
    } catch (error) {
      handleHttpError(error as Error, t("tournament.results.uploadError"));
    } finally {
      setUploading(false);
    }
  };
  const submit = async () => {
    if (!chartId()) return;
    setSubmitting(true);
    try {
      await submitResult(id(), {
        chart_id: chartId()!,
        score: Number(score()),
        accuracy_millionths: Math.round(Number(accuracy()) * 1_000_000),
        max_combo: Number(combo()) || 0,
        full_combo: fullCombo(),
        all_perfect: allPerfect(),
        judgments: {},
        metrics: {},
        played_at: DateTime.now(),
        evidence: evidence(),
      });
      setScore("");
      setAccuracy("");
      setCombo("");
      setEvidence(undefined);
      await refetch();
    } catch (error) {
      handleHttpError(error as Error, t("tournament.results.submitError"));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main class="w-full max-w-6xl mx-auto px-4 py-8 space-y-8">
      <div>
        <h1 class="text-2xl font-bold">{t("tournament.results.title")}</h1>
        <p class="opacity-60 mt-1">{t("tournament.results.subtitle")}</p>
      </div>
      <Card contentClass="p-5">
        <div class="grid md:grid-cols-2 xl:grid-cols-4 gap-4">
          <Select
            size="sm"
            label={t("tournament.results.chart")}
            value={chartId() ? [String(chartId())] : []}
            onValueChange={(event) => {
              const value = event.value[0];
              setChartId(value ? Number(value) : undefined);
            }}
            items={(charts() ?? []).map((chart) => ({
              value: String(chart.id),
              label: `${chart.title} [${chart.difficulty}]`,
            }))}
          />
          <Input
            size="sm"
            title={t("tournament.results.score")}
            type="number"
            min="0"
            value={score()}
            onInput={(event) => setScore(event.currentTarget.value)}
          />
          <Input
            size="sm"
            title={t("tournament.results.accuracy")}
            type="number"
            min="0"
            max="100"
            step="0.000001"
            value={accuracy()}
            onInput={(event) => setAccuracy(event.currentTarget.value)}
          />
          <Input
            size="sm"
            title={t("tournament.results.combo")}
            type="number"
            min="0"
            value={combo()}
            onInput={(event) => setCombo(event.currentTarget.value)}
          />
        </div>
        <div class="flex flex-wrap items-center gap-4 mt-5">
          <label class="flex items-center gap-2">
            <input
              type="checkbox"
              checked={fullCombo()}
              onChange={(event) => setFullCombo(event.currentTarget.checked)}
            />{" "}
            FC
          </label>
          <label class="flex items-center gap-2">
            <input
              type="checkbox"
              checked={allPerfect()}
              onChange={(event) => setAllPerfect(event.currentTarget.checked)}
            />{" "}
            AP
          </label>
          <label class="btn btn-md cursor-pointer" classList={{ "btn-disabled": uploading() }}>
            <span class="icon-[fluent--image-add-20-regular] w-5 h-5" />
            <span>{evidence() ? t("tournament.results.evidenceReady") : t("tournament.results.evidence")}</span>
            <input
              class="hidden"
              type="file"
              accept="image/*"
              onChange={(event) => upload(event.currentTarget.files?.[0])}
            />
          </label>
          <span class="flex-1" />
          <Button
            level="primary"
            loading={submitting()}
            disabled={!chartId() || !score() || !accuracy()}
            onClick={submit}
          >
            <span class="icon-[fluent--send-20-regular] w-5 h-5" />
            {t("tournament.results.submit")}
          </Button>
        </div>
      </Card>

      <section>
        <h2 class="text-lg font-bold mb-3">{t("tournament.results.history")}</h2>
        <div class="overflow-x-auto border border-layer-content/15 rounded-lg">
          <table class="w-full text-sm">
            <thead class="bg-layer-content/5 text-left">
              <tr>
                <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider">
                  {t("tournament.results.chart")}
                </th>
                <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider">
                  {t("tournament.results.score")}
                </th>
                <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider">
                  {t("tournament.results.accuracy")}
                </th>
                <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider">
                  {t("tournament.results.status")}
                </th>
                <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider">
                  {t("tournament.results.playedAt")}
                </th>
              </tr>
            </thead>
            <tbody>
              <For
                each={results()}
                fallback={
                  <tr>
                    <td class="p-8" colSpan="5">
                      <div class="flex flex-col items-center justify-center gap-2 opacity-40">
                        <span class="icon-[fluent--document-dismiss-20-regular] w-8 h-8" />
                        <span>{t("tournament.results.empty")}</span>
                      </div>
                    </td>
                  </tr>
                }
              >
                {(result) => (
                  <tr class="border-t border-layer-content/10 hover:bg-layer-content/5 transition-colors">
                    <td class="p-3">
                      {charts()?.find((chart) => chart.id === result.chart_id)?.title ?? result.chart_id}
                    </td>
                    <td class="p-3 font-mono tabular-nums">{result.score.toLocaleString()}</td>
                    <td class="p-3 font-mono tabular-nums">{(result.accuracy_millionths / 1_000_000).toFixed(4)}%</td>
                    <td class="p-3">
                      <span
                        class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium"
                        classList={{
                          "bg-success/10 text-success": result.status === "approved",
                          "bg-warning/10 text-warning": result.status === "pending",
                          "bg-error/10 text-error": result.status === "rejected",
                          "bg-layer-content/10 opacity-50": result.status === "voided",
                        }}
                      >
                        <span
                          class="w-1.5 h-1.5 rounded-full"
                          classList={{
                            "bg-success": result.status === "approved",
                            "bg-warning": result.status === "pending",
                            "bg-error": result.status === "rejected",
                            "bg-layer-content/40": result.status === "voided",
                          }}
                        />
                        {t(`tournament.results.states.${result.status}`)}
                      </span>
                    </td>
                    <td class="p-3 opacity-60 text-xs">{result.played_at.toLocaleString(DateTime.DATETIME_SHORT)}</td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </div>
      </section>
    </main>
  );
}
