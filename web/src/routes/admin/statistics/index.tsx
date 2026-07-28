import { usePlatformInfo, usePlatformStatistics } from "@api/platform";
import Spin from "@assets/animates/spin";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { For, Show } from "solid-js";

export default function () {
  const statistics = usePlatformStatistics();
  const platformInfo = usePlatformInfo();
  const items = () => [
    {
      icon: "icon-[fluent--trophy-20-regular]",
      label: t("tournament.title"),
      value: statistics.data?.tournaments.total ?? 0,
    },
    {
      icon: "icon-[fluent--play-circle-20-regular]",
      label: t("tournament.lifecycle.running"),
      value: statistics.data?.tournaments.active ?? 0,
    },
    {
      icon: "icon-[fluent--person-add-20-regular]",
      label: t("tournament.registration.title"),
      value: statistics.data?.registrations ?? 0,
    },
    {
      icon: "icon-[fluent--music-note-2-20-regular]",
      label: t("tournament.pool.charts"),
      value: statistics.data?.charts ?? 0,
    },
    {
      icon: "icon-[fluent--clipboard-data-bar-20-regular]",
      label: t("tournament.results.title"),
      value: statistics.data?.results.total ?? 0,
    },
    {
      icon: "icon-[fluent--checkmark-starburst-20-regular]",
      label: t("tournament.results.states.approved"),
      value: statistics.data?.results.approved ?? 0,
    },
  ];

  return (
    <>
      <Title page={t("platform.statistics.title")} route="/admin/statistics" />
      <main class="w-full max-w-6xl mx-auto p-4 lg:p-8 space-y-8">
        <section>
          <h1 class="text-2xl font-bold">{platformInfo.data?.name || "Rhythm Arena"}</h1>
          <p class="opacity-60 mt-1">{t("platform.statistics.title")}</p>
        </section>
        <Show when={!statistics.isLoading && statistics.data} fallback={<Spin width={24} height={24} />}>
          <section class="grid sm:grid-cols-2 lg:grid-cols-3 border border-layer-content/15 rounded-lg overflow-hidden">
            <For each={items()}>
              {(item) => (
                <div class="p-6 border-b border-e border-layer-content/15 last:border-b-0">
                  <div class="flex items-center gap-2 text-sm opacity-60">
                    <span class={`${item.icon} w-5 h-5`} />
                    <span>{item.label}</span>
                  </div>
                  <div class="text-3xl font-bold mt-3">{item.value.toLocaleString()}</div>
                </div>
              )}
            </For>
          </section>
        </Show>
      </main>
    </>
  );
}
