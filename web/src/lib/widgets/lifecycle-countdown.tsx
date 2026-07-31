import type { Tournament } from "@models/tournament";
import { t } from "@storage/theme";
import { DateTime } from "luxon";
import { createMemo, createSignal, Match, onCleanup, Show, Switch } from "solid-js";

/**
 * Live lifecycle countdown for a tournament. Computes the remaining time
 * until the next lifecycle stage based on the tournament's schedule, and
 * re-renders every second. Pass `compact` to render a slimmer variant
 * (no card border, header, or date range) suitable for list cards.
 */
export default function LifecycleCountdown(props: { tournament?: Tournament; compact?: boolean }) {
  const [now, setNow] = createSignal(DateTime.now());
  const interval = setInterval(() => setNow(DateTime.now()), 1000);
  onCleanup(() => clearInterval(interval));

  const countdown = createMemo(() => {
    const item = props.tournament;
    if (!item) return null;
    const stages = {
      draft: {
        target: item.registration_at,
        schedule: item.registration_schedule,
        start: item.created_at,
        targetStage: "registration",
      },
      registration: {
        target: item.running_at,
        schedule: item.running_schedule,
        start: item.registration_at,
        targetStage: "running",
      },
      running: {
        target: item.review_at,
        schedule: item.review_schedule,
        start: item.running_at,
        targetStage: "review",
      },
      review: {
        target: item.finished_at,
        schedule: item.finished_schedule,
        start: item.review_at,
        targetStage: "finished",
      },
      finished: null,
      archived: null,
    } as const;
    const stage = stages[item.lifecycle];
    if (!stage) return { kind: "terminal" as const };
    if (stage.schedule !== "scheduled" || !stage.target) {
      return { kind: "manual" as const, targetStage: stage.targetStage };
    }
    const target = stage.target;
    const start = stage.start ?? target;
    if (!target.isValid || !start.isValid || target <= start) {
      return { kind: "unavailable" as const, targetStage: stage.targetStage };
    }
    const remaining = Math.max(0, target.diff(now()).milliseconds);
    const progress = Math.min(
      100,
      Math.max(0, (target.diff(now()).milliseconds / target.diff(start).milliseconds) * 100)
    );
    return {
      kind: remaining > 0 ? ("scheduled" as const) : ("expired" as const),
      targetStage: stage.targetStage,
      target,
      start,
      remaining,
      progress: 100 - progress,
    };
  });

  const formatCountdown = (milliseconds: number) => {
    const totalSeconds = Math.ceil(milliseconds / 1000);
    const days = Math.floor(totalSeconds / 86400);
    const hours = Math.floor((totalSeconds % 86400) / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;
    return { days, hours, minutes, seconds };
  };

  return (
    <Show when={countdown()}>
      {(countdown) => (
        <div
          class={
            props.compact
              ? "space-y-2 mt-3"
              : "border border-layer-content/15 rounded-xl bg-layer-content/[0.03] p-4 space-y-3"
          }
        >
          <Show when={!props.compact}>
            <div class="flex items-center gap-2">
              <span class="icon-[fluent--timer-20-regular] w-5 h-5 text-primary" />
              <strong class="flex-1">{t("tournament.lifecycleCountdown.title")}</strong>
              <span class="text-xs opacity-50">{t(`tournament.lifecycle.${props.tournament!.lifecycle}`)}</span>
            </div>
          </Show>
          <Switch>
            <Match when={countdown().kind === "scheduled"}>
              {(() => {
                const values = () => formatCountdown(countdown().remaining);
                return (
                  <>
                    <div class="flex items-end gap-2">
                      <div class="text-2xl font-black font-mono tabular-nums">
                        {values().days > 0 ? `${values().days}${t("tournament.lifecycleCountdown.days")} ` : ""}
                        {String(values().hours).padStart(2, "0")}:{String(values().minutes).padStart(2, "0")}:
                        {String(values().seconds).padStart(2, "0")}
                      </div>
                      <span class="text-xs opacity-60 pb-1">
                        {t("tournament.lifecycleCountdown.until", {
                          stage: t(`tournament.lifecycle.${countdown().targetStage}`),
                        })}
                      </span>
                    </div>
                    <div class="h-1.5 rounded-full bg-layer-content/10 overflow-hidden">
                      <div
                        class="h-full bg-primary transition-[width] duration-500"
                        style={{ width: `${countdown().progress}%` }}
                      />
                    </div>
                    <Show when={!props.compact}>
                      <div class="flex justify-between text-xs opacity-50">
                        <span>{countdown().start.toLocal().toFormat("yyyy-MM-dd HH:mm")}</span>
                        <span>{countdown().target.toLocal().toFormat("yyyy-MM-dd HH:mm")}</span>
                      </div>
                    </Show>
                  </>
                );
              })()}
            </Match>
            <Match when={countdown().kind === "terminal"}>
              <p class="text-sm opacity-60">{t(`tournament.lifecycleCountdown.${props.tournament!.lifecycle}`)}</p>
            </Match>
            <Match when={countdown().kind === "expired"}>
              <p class="text-sm opacity-60">{t("tournament.lifecycleCountdown.waitingRefresh")}</p>
            </Match>
            <Match when={true}>
              <p class="text-sm opacity-60">
                {t("tournament.lifecycleCountdown.waitingManual", {
                  stage: t(`tournament.lifecycle.${countdown().targetStage}`),
                })}
              </p>
            </Match>
          </Switch>
        </div>
      )}
    </Show>
  );
}
