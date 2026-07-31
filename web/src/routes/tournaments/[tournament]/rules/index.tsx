import { getTournament } from "@api/tournament";
import { useParams } from "@solidjs/router";
import { t } from "@storage/theme";
import Article from "@widgets/article";
import { createResource, Show } from "solid-js";

export default function () {
  const params = useParams();
  const id = () => Number(params.tournament);
  const [tournament] = createResource(id, getTournament);

  return (
    <main class="w-full max-w-4xl mx-auto px-4 py-8 space-y-6">
      <div class="text-center">
        <h1 class="text-2xl lg:text-3xl font-black">{t("tournament.rules.title")}</h1>
        <div class="mt-3 mx-auto w-16 h-0.5 bg-primary/30 rounded-full" />
      </div>
      <Show
        when={tournament()?.rules_visible && tournament()?.rules}
        fallback={
          <div class="min-h-32 flex flex-col items-center justify-center gap-3 opacity-40 border border-dashed border-layer-content/15 rounded-xl">
            <span class="icon-[fluent--document-text-20-regular] w-10 h-10" />
            <span class="text-sm">{t("tournament.rules.empty")}</span>
          </div>
        }
      >
        <div class="bg-layer-content/[0.02] border border-layer-content/10 rounded-xl px-6 lg:px-8 py-4 lg:py-6">
          <Article content={tournament()!.rules!} noExtraPaddings />
        </div>
      </Show>
    </main>
  );
}
