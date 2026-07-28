import { getLeaderboard } from "@api/tournament";
import { useParams } from "@solidjs/router";
import { t } from "@storage/theme";
import { DateTime } from "luxon";
import { createResource, For, Show } from "solid-js";

function rankMedal(rank: number) {
  if (rank === 1) return "#FFD700";
  if (rank === 2) return "#C0C0C0";
  if (rank === 3) return "#CD7F32";
  return undefined;
}

function rankBg(rank: number) {
  if (rank === 1) return "bg-[#FFD700]/10";
  if (rank === 2) return "bg-[#C0C0C0]/10";
  if (rank === 3) return "bg-[#CD7F32]/10";
  return undefined;
}

function rankIcon(rank: number) {
  if (rank === 1) return "icon-[noto--1st-place-medal] w-6 h-6";
  if (rank === 2) return "icon-[noto--2nd-place-medal] w-6 h-6";
  if (rank === 3) return "icon-[noto--3rd-place-medal] w-6 h-6";
  return undefined;
}

export default function (props: { kind: "individual" | "team" }) {
  const params = useParams();
  const id = () => Number(params.tournament);
  const [snapshot] = createResource(
    () => [id(), props.kind] as const,
    async ([value, kind]) => await getLeaderboard(value, kind)
  );

  return (
    <main class="w-full max-w-5xl mx-auto px-4 py-8 space-y-5">
      <div class="flex flex-wrap items-end gap-4">
        <div>
          <h1 class="text-2xl font-bold">{t(`tournament.leaderboard.${props.kind}`)}</h1>
          <p class="opacity-60 mt-1">{t("tournament.leaderboard.subtitle")}</p>
        </div>
        <span class="flex-1" />
        <Show when={snapshot()}>
          {(snap) => (
            <div class="flex items-center gap-2 text-xs opacity-50">
              <span class="icon-[fluent--clock-20-regular] w-4 h-4" />
              <span>{snap().computed_at.toLocaleString(DateTime.DATETIME_SHORT)}</span>
            </div>
          )}
        </Show>
      </div>

      <Show when={snapshot()?.stale}>
        <div class="p-3 border border-warning/40 bg-warning/10 rounded-lg text-warning flex items-center gap-2">
          <span class="icon-[fluent--warning-20-filled] w-5 h-5 shrink-0" />
          <span class="text-sm">{t("tournament.leaderboard.stale")}</span>
        </div>
      </Show>

      <div class="border border-layer-content/15 rounded-lg overflow-hidden">
        {/* Header */}
        <div class="grid grid-cols-[4rem_1fr_auto] gap-3 items-center px-4 py-2.5 bg-layer-content/5 border-b border-layer-content/15">
          <span class="text-xs opacity-50 text-center font-bold">#</span>
          <span class="text-xs opacity-50 font-bold">{t("tournament.registration.displayName")}</span>
          <span class="text-xs opacity-50 font-bold text-right">{t("tournament.results.score")}</span>
        </div>

        <For
          each={snapshot()?.entries}
          fallback={
            <div class="p-12 flex flex-col items-center justify-center gap-4 opacity-40">
              <span class="icon-[fluent--trophy-20-regular] w-16 h-16" />
              <p class="text-sm">{t("tournament.leaderboard.empty")}</p>
            </div>
          }
        >
          {(entry) => (
            <div
              class="grid grid-cols-[4rem_1fr_auto] gap-3 items-center px-4 py-3 border-b border-layer-content/10 last:border-b-0 transition-colors hover:bg-layer-content/5"
              classList={{ [rankBg(entry.rank) || ""]: entry.rank <= 3 }}
            >
              <span
                class="flex items-center justify-center gap-1"
                classList={{
                  "text-xl font-bold": entry.rank > 3,
                  "text-2xl font-black": entry.rank <= 3,
                }}
                style={{ color: rankMedal(entry.rank) }}
              >
                <Show when={rankIcon(entry.rank)} fallback={<span>#{entry.rank}</span>}>
                  {(icon) => <span class={icon()} />}
                </Show>
              </span>
              <strong class="truncate">{entry.name}</strong>
              <span class="font-mono text-lg tabular-nums">{entry.score.toLocaleString()}</span>
            </div>
          )}
        </For>
      </div>
    </main>
  );
}
