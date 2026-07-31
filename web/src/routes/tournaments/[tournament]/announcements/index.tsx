import { getNotifications, getTournament } from "@api/tournament";
import { useParams } from "@solidjs/router";
import { t } from "@storage/theme";
import Article from "@widgets/article";
import { createResource, For, Show } from "solid-js";

export default function () {
  const params = useParams();
  const id = () => Number(params.tournament);
  const [tournament] = createResource(id, getTournament);
  const [notifications] = createResource(id, getNotifications);
  const enabled = () => tournament()?.announcements_visible === true;

  return (
    <main class="w-full max-w-4xl mx-auto px-4 py-8 space-y-6">
      <div class="text-center">
        <h1 class="text-2xl lg:text-3xl font-black">{t("tournament.announcements.title")}</h1>
        <div class="mt-3 mx-auto w-16 h-0.5 bg-primary/30 rounded-full" />
      </div>
      <Show
        when={enabled()}
        fallback={
          <div class="min-h-32 flex flex-col items-center justify-center gap-3 opacity-40 border border-dashed border-layer-content/15 rounded-xl">
            <span class="icon-[fluent--megaphone-loud-20-regular] w-10 h-10" />
            <span class="text-sm">{t("tournament.announcements.empty")}</span>
          </div>
        }
      >
        <For
          each={notifications()}
          fallback={
            <div class="min-h-32 flex flex-col items-center justify-center gap-3 opacity-40 border border-dashed border-layer-content/15 rounded-xl">
              <span class="icon-[fluent--megaphone-loud-20-regular] w-10 h-10" />
              <span class="text-sm">{t("tournament.announcements.empty")}</span>
            </div>
          }
        >
          {(item) => (
            <article class="bg-layer-content/[0.02] border border-layer-content/10 rounded-xl px-6 lg:px-8 py-4 lg:py-6 space-y-3">
              <div class="flex items-center gap-3 border-b border-layer-content/10 pb-3">
                <span class="icon-[fluent--megaphone-loud-20-regular] w-5 h-5 text-primary shrink-0" />
                <h2 class="flex-1 text-lg font-bold">{item.title}</h2>
                <time class="text-xs opacity-50 shrink-0">
                  {item.published_at?.toLocal().toFormat("yyyy-MM-dd HH:mm")}
                </time>
              </div>
              <Article content={item.content} noExtraPaddings />
            </article>
          )}
        </For>
      </Show>
    </main>
  );
}
