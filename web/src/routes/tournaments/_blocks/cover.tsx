import LogoAnimate from "@assets/animates/logo-animate";
import { mediaPath } from "@lib/utils/media";
import type { Tournament } from "@models/tournament";
import { useNavigate } from "@solidjs/router";
import LoadingTips from "@widgets/loading-tips";
import clsx from "clsx";
import { createEffect, createRoot, createSignal, onCleanup, Show, untrack } from "solid-js";
import { createStore } from "solid-js/store";

const tournamentCoverRoot = createRoot(() =>
  createStore<{ preload: Tournament | null; goto: number | null }>({ preload: null, goto: null })
);

export const tournamentCoverStore = tournamentCoverRoot[0];
export const setTournamentCoverStore = tournamentCoverRoot[1];

export default function () {
  const navigate = useNavigate();
  const [expanded, setExpanded] = createSignal(false);
  const timers: Array<ReturnType<typeof setTimeout>> = [];

  createEffect(() => {
    const target = tournamentCoverStore.goto;
    if (!target || expanded()) return;
    untrack(() => {
      setExpanded(true);
      timers.push(
        setTimeout(() => navigate(`/tournaments/${target}`), 700),
        setTimeout(() => setExpanded(false), 1200),
        setTimeout(() => setTournamentCoverStore({ goto: null, preload: null }), 1600)
      );
    });
  });

  onCleanup(() => {
    for (const timer of timers) clearTimeout(timer);
  });

  return (
    <div
      class={clsx(
        "fixed w-full top-0 left-0 overflow-hidden transition-all ease-in-out z-40 duration-500 print:hidden",
        expanded() ? "h-full" : "h-0"
      )}
    >
      <div class="w-screen h-screen relative bg-layer">
        <img
          class={clsx(
            "w-screen h-screen object-cover transition-all ease-out duration-1000",
            expanded() && "scale-110 blur-md"
          )}
          alt={tournamentCoverStore.preload?.name || "Tournament cover"}
          src={mediaPath(tournamentCoverStore.preload?.cover)}
        />
        <div
          class={clsx(
            "absolute inset-0 flex flex-col items-center justify-center transition-all duration-700",
            expanded() ? "bg-layer/80" : "bg-layer/20"
          )}
        >
          <div
            class={clsx(
              "h-40 w-40 flex items-center justify-center transition-all duration-500",
              expanded() ? "opacity-100" : "scale-125 blur-xl opacity-0 rotate-90"
            )}
          >
            <LogoAnimate class="w-full h-full object-contain" />
          </div>
          <div
            class={clsx(
              "mt-8 max-w-xl px-6 text-center overflow-hidden transition-all duration-500 delay-200",
              expanded() ? "max-h-40 opacity-100" : "max-h-0 opacity-0"
            )}
          >
            <h1 class="text-3xl font-bold">{tournamentCoverStore.preload?.name}</h1>
            <p class="mt-3 opacity-60">{tournamentCoverStore.preload?.brief}</p>
          </div>
        </div>
        <Show when={expanded()}>
          <div class="absolute left-6 bottom-4">
            <LoadingTips />
          </div>
        </Show>
      </div>
    </div>
  );
}
