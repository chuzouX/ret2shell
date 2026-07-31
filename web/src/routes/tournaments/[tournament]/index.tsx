import { handleHttpError } from "@api";
import {
  createTeam,
  getMyRegistration,
  getTeams,
  getTournament,
  joinTeam,
  registerTournament,
  withdrawRegistration,
} from "@api/tournament";
import bgTournamentDefault from "@assets/imgs/bg-game-default.webp";
import { mediaPath } from "@lib/utils/media";
import { useParams } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { t } from "@storage/theme";
import Article from "@widgets/article";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Input from "@widgets/input";
import LifecycleCountdown from "@widgets/lifecycle-countdown";
import Link from "@widgets/link";
import Picture from "@widgets/picture";
import Tag from "@widgets/tag";
import { createMemo, createResource, createSignal, For, Show } from "solid-js";

export default function () {
  const params = useParams();
  const id = () => Number(params.tournament);
  const [tournament] = createResource(id, getTournament);
  const [teams, { refetch: refetchTeams }] = createResource(id, getTeams);
  const [registration, { refetch: refetchRegistration }] = createResource(
    () => accountStore.token && id(),
    async (value) => (value ? await getMyRegistration(Number(value)) : null)
  );
  const [displayName, setDisplayName] = createSignal("");
  const [teamName, setTeamName] = createSignal("");
  const [busy, setBusy] = createSignal(false);

  const registrationOpen = createMemo(() => tournament()?.lifecycle === "registration");

  const lifecycleLevel = () => {
    const l = tournament()?.lifecycle;
    if (l === "running") return "success" as const;
    if (l === "review" || l === "finished") return "warning" as const;
    if (l === "archived") return "error" as const;
    return "info" as const;
  };

  const run = async (action: () => Promise<unknown>) => {
    setBusy(true);
    try {
      await action();
      await Promise.all([refetchRegistration(), refetchTeams()]);
    } catch (error) {
      handleHttpError(error as Error, t("tournament.errors.action"));
    } finally {
      setBusy(false);
    }
  };

  return (
    <main class="flex-1 flex flex-col lg:flex-row-reverse">
      {/* Sidebar — right side on desktop */}
      <aside class="lg:w-2/5 lg:max-h-[calc(100vh-4rem)] lg:sticky lg:top-16 flex flex-col p-3 lg:p-6 gap-4 overflow-y-auto">
        {/* Cover image with title overlay */}
        <Card
          class="aspect-video w-full overflow-hidden relative bg-layer-content/5"
          contentClass="relative w-full h-full"
        >
          <Picture
            class="w-full h-full"
            src={tournament()?.cover ? mediaPath(tournament()?.cover) : bgTournamentDefault}
            alt={tournament()?.name || t("tournament.title")}
          />
          <Show when={tournament()}>
            {(item) => (
              <>
                <Tag class="absolute top-3 right-3 z-10" level={lifecycleLevel()}>
                  {t(`tournament.lifecycle.${item().lifecycle}`)}
                </Tag>
                <div class="absolute bottom-0 left-0 right-0 bg-layer/50 backdrop-blur-sm p-4 space-y-1">
                  <h1 class="text-2xl font-black">{item().name}</h1>
                  <p class="opacity-80 text-sm">{item().brief || t("tournament.noBrief")}</p>
                  <Show when={item().start_at || item().end_at}>
                    <p class="text-sm text-info flex flex-wrap gap-x-2">
                      <span>{item().start_at?.toFormat("yyyy-MM-dd HH:mm") || "--"}</span>
                      <span class="opacity-50">—</span>
                      <span>{item().end_at?.toFormat("yyyy-MM-dd HH:mm") || "--"}</span>
                    </p>
                  </Show>
                </div>
              </>
            )}
          </Show>
        </Card>

        <LifecycleCountdown tournament={tournament()} />

        {/* Mode / Evidence / Team size tags */}
        <div class="flex flex-wrap gap-2 px-1">
          <Tag level="info">{t(`tournament.mode.${tournament()?.competition_mode}`)}</Tag>
          <Tag level="success">{t(`tournament.evidence.${tournament()?.evidence_policy}`)}</Tag>
          <Show when={tournament()?.competition_mode !== "individual"}>
            <Tag level="warning">
              {tournament()?.team_size_min}—{tournament()?.team_size_max}
            </Tag>
          </Show>
        </div>

        {/* Registration / CTA */}
        <div class="border-t border-layer-content/10 pt-4 px-1">
          <Show
            when={accountStore.token}
            fallback={
              <Link
                href={`/account/login?redirect=${encodeURIComponent(`/tournaments/${id()}`)}`}
                level="primary"
                class="w-full"
              >
                <span class="icon-[fluent--person-20-regular] w-5 h-5" />
                <span>{t("tournament.registration.loginRequired")}</span>
              </Link>
            }
          >
            <Show
              when={registration()}
              fallback={
                <div class="space-y-2">
                  <Input
                    noLabel
                    placeholder={t("tournament.registration.displayName")}
                    value={displayName()}
                    onInput={(event) => setDisplayName(event.currentTarget.value)}
                  />
                  <Button
                    class="w-full"
                    level="primary"
                    loading={busy()}
                    disabled={!registrationOpen()}
                    onClick={() => run(async () => await registerTournament(id(), displayName() || undefined))}
                  >
                    <span class="icon-[fluent--person-add-20-regular] w-5 h-5" />
                    <span>{t("tournament.registration.join")}</span>
                  </Button>
                </div>
              }
            >
              {(record) => (
                <div class="space-y-3">
                  <Card contentClass="p-3 flex items-center gap-3">
                    <span class="icon-[fluent--person-20-filled] w-8 h-8 text-primary shrink-0" />
                    <div class="flex-1 min-w-0">
                      <div class="font-bold truncate">{record().display_name}</div>
                      <div class="text-xs opacity-60">{t(`tournament.registration.status.${record().status}`)}</div>
                    </div>
                    <Show when={record().status === "approved"}>
                      <span class="icon-[fluent--checkmark-circle-20-filled] text-success w-5 h-5 shrink-0" />
                    </Show>
                  </Card>
                  <div class="grid grid-cols-2 gap-2">
                    <Link href={`/tournaments/${id()}/results`} level="primary">
                      <span class="icon-[fluent--document-checkmark-20-regular] w-5 h-5" />
                      <span>{t("tournament.nav.results")}</span>
                    </Link>
                    <Button
                      ghost
                      level="warning"
                      loading={busy()}
                      disabled={!registrationOpen()}
                      onClick={() => run(async () => await withdrawRegistration(id()))}
                    >
                      {t("tournament.registration.withdraw")}
                    </Button>
                  </div>
                </div>
              )}
            </Show>
          </Show>
        </div>
      </aside>

      {/* Main content — left side on desktop */}
      <div class="flex-1 min-w-0 flex flex-col p-3 lg:p-6 gap-2">
        {/* Introduction */}
        <section id="intro" class="py-8">
          <div class="text-center mb-8">
            <h2 class="text-2xl lg:text-3xl font-black">{t("tournament.introduction")}</h2>
            <div class="mt-3 mx-auto w-16 h-0.5 bg-primary/30 rounded-full" />
          </div>
          <Show
            when={tournament()?.description}
            fallback={
              <div class="min-h-32 flex flex-col items-center justify-center gap-3 opacity-40 border border-dashed border-layer-content/15 rounded-xl">
                <span class="icon-[fluent--document-text-20-regular] w-10 h-10" />
                <span class="text-sm">{t("tournament.noDescription")}</span>
              </div>
            }
          >
            <div class="max-w-4xl mx-auto bg-layer-content/[0.02] border border-layer-content/10 rounded-xl px-6 lg:px-8 py-4 lg:py-6">
              <Article content={tournament()!.description!} noExtraPaddings />
            </div>
          </Show>
        </section>

        {/* Teams */}
        <Show when={tournament()?.competition_mode !== "individual"}>
          <section id="teams" class="py-8 border-t border-layer-content/10">
            <div class="text-center mb-8">
              <h2 class="text-2xl lg:text-3xl font-black">{t("tournament.teams.title")}</h2>
              <div class="mt-3 mx-auto w-16 h-0.5 bg-primary/30 rounded-full" />
            </div>
            <div class="flex flex-wrap items-center gap-2 mb-5">
              <span class="flex-1" />
              <Show when={registration()}>
                <Input
                  noLabel
                  size="sm"
                  placeholder={t("tournament.teams.name")}
                  value={teamName()}
                  onInput={(event) => setTeamName(event.currentTarget.value)}
                />
                <Button
                  size="sm"
                  level="primary"
                  disabled={!teamName().trim() || !registrationOpen()}
                  onClick={() =>
                    run(async () => {
                      await createTeam(id(), teamName().trim());
                      setTeamName("");
                    })
                  }
                >
                  <span class="icon-[fluent--add-20-regular] w-5 h-5" />
                  {t("tournament.teams.create")}
                </Button>
              </Show>
            </div>
            <div class="grid sm:grid-cols-2 xl:grid-cols-3 gap-2">
              <For
                each={teams()}
                fallback={
                  <div class="col-span-full min-h-20 flex flex-col items-center justify-center gap-2 opacity-40 border border-dashed border-layer-content/15 rounded-xl">
                    <span class="icon-[fluent--people-team-20-regular] w-8 h-8" />
                    <span class="text-sm">{t("tournament.teams.empty")}</span>
                  </div>
                }
              >
                {(team) => (
                  <div class="group p-3 border border-layer-content/10 hover:border-layer-content/20 rounded-lg flex items-center gap-2.5 min-w-0 transition-all duration-200 bg-layer-content/[0.02] hover:bg-layer-content/[0.05]">
                    <span class="icon-[fluent--shield-person-20-regular] w-5 h-5 text-primary shrink-0" />
                    <strong class="truncate flex-1 text-sm">{team.name}</strong>
                    <Show when={registration()}>
                      <Button
                        size="sm"
                        ghost
                        disabled={!registrationOpen()}
                        onClick={() => run(async () => await joinTeam(id(), team.id))}
                      >
                        {t("tournament.teams.join")}
                      </Button>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </section>
        </Show>
      </div>
    </main>
  );
}
