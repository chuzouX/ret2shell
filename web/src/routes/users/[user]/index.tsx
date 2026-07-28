import { getUser, getUserTeams, getUserTournamentStats } from "@api/user";
import SidebarLayout from "@blocks/sidebar-layout";
import type { User } from "@models/user";
import { A, useNavigate, useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Article from "@widgets/article";
import LoadingTips from "@widgets/loading-tips";
import { createResource, For, Show } from "solid-js";
import Sidebar from "./_blocks/sidebar";

export default function () {
  const params = useParams();
  const navigate = useNavigate();
  const userId = () => Number.parseInt(params.user ?? "", 10);
  if (!userId()) {
    navigate("/error/404", { replace: true });
  }

  const [user] = createResource(userId, getUser);
  const [teams] = createResource(userId, getUserTeams);
  const [statistics] = createResource(userId, (id) => getUserTournamentStats(id));

  return (
    <>
      <Title page={user()?.nickname ?? t("user.title")} route={`/users/${userId()}`} />
      <SidebarLayout leftBar={() => <Sidebar user={(user() as User | undefined) ?? null} loading={user.loading} />}>
        <main class="w-full max-w-5xl mx-auto px-4 py-8 space-y-8">
          <Show when={!user.loading} fallback={<LoadingTips />}>
            <section>
              <h1 class="text-2xl font-bold">{user()?.nickname}</h1>
              <p class="opacity-60 mt-1">@{user()?.account}</p>
            </section>

            <section class="grid grid-cols-2 gap-px bg-layer-content/15 border border-layer-content/15 rounded-lg overflow-hidden">
              <div class="bg-layer p-5">
                <div class="text-sm opacity-60">{t("tournament.registration.title")}</div>
                <div class="text-3xl font-bold mt-2">{statistics()?.registrations ?? 0}</div>
              </div>
              <div class="bg-layer p-5">
                <div class="text-sm opacity-60">{t("tournament.results.states.approved")}</div>
                <div class="text-3xl font-bold mt-2">{statistics()?.approved_results ?? 0}</div>
              </div>
            </section>

            <Show when={user()?.description}>
              <section class="prose max-w-none">
                <Article content={user()!.description!} />
              </section>
            </Show>

            <section class="space-y-3">
              <h2 class="text-lg font-bold">{t("tournament.teams.title")}</h2>
              <div class="divide-y divide-layer-content/10 border-y border-layer-content/15">
                <For each={teams()} fallback={<p class="py-5 opacity-60">{t("tournament.teams.empty")}</p>}>
                  {(team) => (
                    <A
                      href={`/tournaments/${team.tournament_id}`}
                      class="py-4 flex items-center gap-3 hover:text-primary"
                    >
                      <span class="icon-[fluent--people-team-20-regular] w-5 h-5" />
                      <strong class="truncate">{team.name}</strong>
                      <span class="icon-[fluent--chevron-right-20-regular] w-5 h-5 ms-auto" />
                    </A>
                  )}
                </For>
              </div>
            </section>
          </Show>
        </main>
      </SidebarLayout>
    </>
  );
}
