import { handleHttpError } from "@api";
import { uploadMedia } from "@api/media";
import { createTournament, getTournaments } from "@api/tournament";
import LogoAnimate from "@assets/animates/logo-animate";
import bgTournamentDefault from "@assets/imgs/bg-game-default.webp";
import { mediaPath } from "@lib/utils/media";
import type {
  CompetitionMode,
  EvidencePolicy,
  LeaderboardVisibility,
  Tournament,
  TournamentLifecycle,
} from "@models/tournament";
import { useSearchParams } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Divider from "@widgets/divider";
import Input from "@widgets/input";
import LifecycleCountdown from "@widgets/lifecycle-countdown";
import Picture from "@widgets/picture";
import Popover from "@widgets/popover";
import Select from "@widgets/select";
import Tag from "@widgets/tag";
import clsx from "clsx";
import { createMemo, createResource, createSignal, For, Show } from "solid-js";
import { setTournamentCoverStore } from "./_blocks/cover";

const pageSize = 5;

function lifecycleLevel(lifecycle: TournamentLifecycle): "info" | "success" | "warning" | "error" {
  if (lifecycle === "registration") return "info";
  if (lifecycle === "running") return "success";
  if (lifecycle === "review" || lifecycle === "finished") return "warning";
  if (lifecycle === "archived") return "error";
  return "info";
}

function lifecycleDot(lifecycle: TournamentLifecycle) {
  if (lifecycle === "registration") return "bg-info";
  if (lifecycle === "running") return "bg-success";
  if (lifecycle === "review" || lifecycle === "finished") return "bg-warning";
  if (lifecycle === "archived") return "bg-error";
  return "bg-layer-content/40";
}

export default function () {
  const [searchParams, setSearchParams] = useSearchParams();
  const [tournaments, { refetch }] = createResource(getTournaments);
  const [page, setPage] = createSignal(1);
  const [name, setName] = createSignal("");
  const [brief, setBrief] = createSignal("");
  const [competitionMode, setCompetitionMode] = createSignal<CompetitionMode>("both");
  const [evidencePolicy, setEvidencePolicy] = createSignal<EvidencePolicy>("optional");
  const [leaderboardVisibility, setLeaderboardVisibility] = createSignal<LeaderboardVisibility>("live");
  const [teamSizeMin, setTeamSizeMin] = createSignal("1");
  const [teamSizeMax, setTeamSizeMax] = createSignal("5");
  const [coverFile, setCoverFile] = createSignal<File>();
  const [uploadingCover, setUploadingCover] = createSignal(false);
  const [creating, setCreating] = createSignal(false);

  const featured = createMemo(() => {
    const current = (tournaments() ?? []).filter(
      (item) => item.lifecycle !== "finished" && item.lifecycle !== "archived"
    );
    return current.length > 0 ? current : (tournaments() ?? []).slice(0, pageSize);
  });
  const history = createMemo(() =>
    (tournaments() ?? []).filter((item) => item.lifecycle === "finished" || item.lifecycle === "archived")
  );
  const totalPages = createMemo(() => Math.max(1, Math.ceil(featured().length / pageSize)));
  const visibleFeatured = createMemo(() => featured().slice((page() - 1) * pageSize, page() * pageSize));
  const showCreate = () => searchParams.create === "true";
  const selectedTournament = createMemo(() => {
    const selected = Number.parseInt(String(searchParams.selected ?? ""), 10);
    return visibleFeatured().find((item) => item.id === selected) ?? visibleFeatured()[0];
  });

  const selectTournament = (item: Tournament) => {
    setSearchParams({ selected: String(item.id), create: undefined });
  };

  const moveSelection = (offset: number) => {
    const items = visibleFeatured();
    if (items.length === 0) return;
    const current = items.findIndex((item) => item.id === selectedTournament()?.id);
    selectTournament(items[(current + offset + items.length) % items.length]);
  };

  const enterTournament = (item: Tournament) => {
    setTournamentCoverStore({ preload: item, goto: item.id });
  };

  const create = async () => {
    if (!name().trim()) return;
    setCreating(true);
    try {
      let cover: string | undefined;
      if (coverFile()) {
        setUploadingCover(true);
        cover = (await uploadMedia(coverFile()!)).hash;
        setUploadingCover(false);
      }
      const created = await createTournament({
        name: name().trim(),
        brief: brief().trim(),
        competition_mode: competitionMode(),
        evidence_policy: evidencePolicy(),
        leaderboard_visibility: leaderboardVisibility(),
        team_size_min: Math.max(1, Number(teamSizeMin()) || 1),
        team_size_max: Math.max(1, Number(teamSizeMax()) || 1),
        cover,
      });
      setName("");
      setBrief("");
      setCoverFile(undefined);
      await refetch();
      enterTournament(created);
    } catch (error) {
      handleHttpError(error as Error, t("tournament.errors.create"));
    } finally {
      setCreating(false);
    }
  };

  return (
    <>
      <Title page={t("tournament.title")} route="/tournaments" />
      <main class="flex-1 relative">
        <div class="lg:absolute lg:h-full lg:w-full overflow-y-auto lg:snap-mandatory lg:snap-y">
          <section class="min-h-[calc(100vh-4rem)] lg:h-full lg:snap-center flex flex-col lg:flex-row relative">
            <aside class="w-1/4 hidden lg:flex flex-col items-end justify-start py-20 space-y-2">
              <Show when={accountStore.token}>
                <Button
                  level="primary"
                  class="w-4/5"
                  justify="start"
                  onClick={() => setSearchParams({ selected: undefined, create: "true" })}
                >
                  <span class="icon-[fluent--add-20-regular] w-5 h-5 opacity-60" />
                  <span>{t("tournament.actions.create")}</span>
                </Button>
              </Show>
              <Divider class="w-4/5" />
              <Button ghost class="w-4/5" disabled={page() <= 1} onClick={() => setPage((value) => value - 1)}>
                <span class="icon-[fluent--chevron-double-up-20-regular] w-5 h-5 opacity-60" />
              </Button>
              <Divider class="w-4/5" />
              <For
                each={visibleFeatured()}
                fallback={
                  <Button ghost disabled class="w-4/5" justify="start">
                    <span class="icon-[fluent--music-note-2-20-regular] w-5 h-5" />
                    <span>{t("tournament.empty")}</span>
                  </Button>
                }
              >
                {(item) => (
                  <Button
                    ghost
                    active={selectedTournament()?.id === item.id && !showCreate()}
                    class="w-4/5"
                    justify="start"
                    onClick={() => selectTournament(item)}
                  >
                    <span
                      class={clsx(
                        selectedTournament()?.id === item.id
                          ? "icon-[fluent--music-note-2-20-filled] text-primary"
                          : "icon-[fluent--music-note-2-20-regular] opacity-60",
                        "w-5 h-5 shrink-0"
                      )}
                    />
                    <span class="flex-1 text-start truncate">{item.name}</span>
                    <span class={clsx("w-2 h-2 rounded-full shrink-0", lifecycleDot(item.lifecycle))} />
                  </Button>
                )}
              </For>
              <Divider class="w-4/5" />
              <Button
                ghost
                class="w-4/5"
                disabled={page() >= totalPages()}
                onClick={() => setPage((value) => value + 1)}
              >
                <span class="icon-[fluent--chevron-double-down-20-regular] w-5 h-5 opacity-60" />
              </Button>
              <Divider class="w-4/5" />
              <div class="flex-1" />
              <Divider class="w-4/5" />
              <Button
                ghost
                class="w-4/5"
                justify="start"
                onClick={() => document.getElementById("past-tournaments")?.scrollIntoView({ behavior: "smooth" })}
              >
                <span class="icon-[fluent--chevron-double-down-20-regular] w-5 h-5" />
                <span>{t("tournament.past")}</span>
              </Button>
              <Divider class="w-4/5" />
            </aside>

            <Card class="block lg:hidden mx-3 mt-3" contentClass="p-2 flex flex-row gap-2">
              <Button ghost square disabled={visibleFeatured().length < 2} onClick={() => moveSelection(-1)}>
                <span class="icon-[fluent--chevron-double-left-20-regular] w-5 h-5" />
              </Button>
              <Popover
                popContentClass="pt-2 flex flex-col"
                ghost
                class="flex-1"
                btnContent={<span class="truncate">{selectedTournament()?.name || t("tournament.empty")}</span>}
              >
                <Card class="w-[80vw]" contentClass="p-2 flex flex-col gap-2">
                  <For each={visibleFeatured()}>
                    {(item) => (
                      <Button
                        ghost
                        active={selectedTournament()?.id === item.id}
                        justify="start"
                        onClick={() => selectTournament(item)}
                      >
                        <span class="icon-[fluent--music-note-2-20-regular] w-5 h-5" />
                        <span class="flex-1 text-start truncate">{item.name}</span>
                        <span class={clsx("w-2 h-2 rounded-full", lifecycleDot(item.lifecycle))} />
                      </Button>
                    )}
                  </For>
                </Card>
              </Popover>
              <Button ghost square disabled={visibleFeatured().length < 2} onClick={() => moveSelection(1)}>
                <span class="icon-[fluent--chevron-double-right-20-regular] w-5 h-5" />
              </Button>
            </Card>

            <div class="hidden lg:block w-16" />
            <div class="flex-1 p-3 lg:p-12 flex flex-col items-center lg:justify-center lg:items-start">
              <Show
                when={!showCreate()}
                fallback={
                  <Card class="w-full max-w-2xl" contentClass="p-6 lg:p-9 space-y-5">
                    <div class="flex items-center gap-3">
                      <span class="icon-[fluent--trophy-20-filled] w-8 h-8 text-primary" />
                      <div>
                        <h1 class="text-2xl font-bold">{t("tournament.actions.create")}</h1>
                        <p class="opacity-60 mt-1">{t("tournament.listSubtitle")}</p>
                      </div>
                    </div>
                    <Input
                      title={t("tournament.fields.name")}
                      value={name()}
                      onInput={(event) => setName(event.currentTarget.value)}
                    />
                    <Input
                      title={t("tournament.fields.brief")}
                      value={brief()}
                      onInput={(event) => setBrief(event.currentTarget.value)}
                    />
                    <Select
                      label={t("tournament.fields.mode")}
                      value={[competitionMode()]}
                      onValueChange={(e) => setCompetitionMode(e.value[0] as CompetitionMode)}
                      items={[
                        { label: t("tournament.mode.individual"), value: "individual" },
                        { label: t("tournament.mode.team"), value: "team" },
                        { label: t("tournament.mode.both"), value: "both" },
                      ]}
                    />
                    <Select
                      label={t("tournament.fields.evidence")}
                      value={[evidencePolicy()]}
                      onValueChange={(e) => setEvidencePolicy(e.value[0] as EvidencePolicy)}
                      items={[
                        { label: t("tournament.evidence.required"), value: "required" },
                        { label: t("tournament.evidence.optional"), value: "optional" },
                        { label: t("tournament.evidence.disabled"), value: "disabled" },
                      ]}
                    />
                    <Select
                      label={t("tournament.fields.visibility")}
                      value={[leaderboardVisibility()]}
                      onValueChange={(e) => setLeaderboardVisibility(e.value[0] as LeaderboardVisibility)}
                      items={[
                        { label: t("tournament.visibility.live"), value: "live" },
                        { label: t("tournament.visibility.frozen"), value: "frozen" },
                        { label: t("tournament.visibility.after_end"), value: "after_end" },
                      ]}
                    />
                    <Show when={competitionMode() !== "individual"}>
                      <div class="grid grid-cols-2 gap-3">
                        <Input
                          title={t("tournament.fields.teamSizeMin")}
                          type="number"
                          size="sm"
                          min="1"
                          value={teamSizeMin()}
                          onInput={(event) => setTeamSizeMin(event.currentTarget.value)}
                        />
                        <Input
                          title={t("tournament.fields.teamSizeMax")}
                          type="number"
                          size="sm"
                          min="1"
                          value={teamSizeMax()}
                          onInput={(event) => setTeamSizeMax(event.currentTarget.value)}
                        />
                      </div>
                    </Show>
                    <div class="flex flex-col space-y-1">
                      <span class="label">{t("tournament.fields.cover")}</span>
                      <label
                        class="btn btn-md cursor-pointer w-fit"
                        classList={{ "btn-disabled": uploadingCover() || creating() }}
                      >
                        <span
                          class="icon-[fluent--image-add-20-regular] w-5 h-5"
                          classList={{ "animate-spin": uploadingCover() }}
                        />
                        <span class="max-w-48 truncate">
                          {uploadingCover()
                            ? t("general.actions.upload.status.pending")
                            : coverFile()?.name || t("general.actions.upload.title")}
                        </span>
                        <input
                          class="hidden"
                          type="file"
                          accept="image/*"
                          onChange={(event) => setCoverFile(event.currentTarget.files?.[0])}
                        />
                      </label>
                    </div>
                    <div class="flex justify-end gap-2">
                      <Button ghost onClick={() => setSearchParams({ create: undefined })}>
                        {t("general.actions.cancel.title")}
                      </Button>
                      <Button level="primary" loading={creating()} disabled={!name().trim()} onClick={create}>
                        <span class="icon-[fluent--add-20-regular] w-5 h-5" />
                        <span>{t("tournament.actions.create")}</span>
                      </Button>
                    </div>
                  </Card>
                }
              >
                <Card
                  class="aspect-video w-full lg:w-11/12 rounded-b-none lg:rounded-b-lg border-b-0 lg:border-b overflow-hidden relative"
                  contentClass="relative"
                >
                  <Show
                    when={selectedTournament()}
                    fallback={
                      <div class="w-full h-full flex items-center justify-center bg-layer-content/5">
                        <LogoAnimate class="w-1/3 h-1/3 grayscale opacity-40" />
                      </div>
                    }
                  >
                    {(item) => (
                      <Picture
                        class="aspect-video"
                        src={item().cover ? mediaPath(item().cover) : bgTournamentDefault}
                        alt={item().name}
                      />
                    )}
                  </Show>
                  <Show when={selectedTournament()}>
                    {(item) => (
                      <>
                        <Tag class="absolute top-2 right-2" level={lifecycleLevel(item().lifecycle)}>
                          <span>{t(`tournament.lifecycle.${item().lifecycle}`)}</span>
                        </Tag>
                        <button
                          type="button"
                          class="absolute inset-0 w-full h-full cursor-pointer"
                          aria-label={item().name}
                          onClick={() => enterTournament(item())}
                        />
                      </>
                    )}
                  </Show>
                </Card>
                <Card
                  class="w-full lg:w-3/5 relative lg:translate-y-8 lg:translate-x-2/3 rounded-t-none lg:rounded-t-lg border-t-0 lg:border-t"
                  contentClass="p-6 lg:px-9 flex items-center gap-6"
                >
                  <span class="hidden lg:block icon-[fluent--music-note-2-20-filled] w-14 h-14 text-primary shrink-0" />
                  <div class="min-w-0 flex-1">
                    <h1 class="text-xl font-bold truncate">{selectedTournament()?.name || t("tournament.empty")}</h1>
                    <p class="opacity-60 mt-1 line-clamp-2">{selectedTournament()?.brief || t("tournament.noBrief")}</p>
                    <Show when={selectedTournament()?.start_at || selectedTournament()?.end_at}>
                      <p class="text-sm text-info mt-3">
                        {selectedTournament()?.start_at?.toFormat("yyyy-MM-dd HH:mm") || "--"}
                        <span class="mx-2">-</span>
                        {selectedTournament()?.end_at?.toFormat("yyyy-MM-dd HH:mm") || "--"}
                      </p>
                    </Show>
                    <LifecycleCountdown tournament={selectedTournament()} compact />
                  </div>
                  <Show when={selectedTournament()}>
                    {(item) => (
                      <Button
                        square
                        level="primary"
                        title={t("general.actions.goto.title")}
                        onClick={() => enterTournament(item())}
                      >
                        <span class="icon-[fluent--chevron-double-right-20-regular] w-5 h-5" />
                      </Button>
                    )}
                  </Show>
                </Card>
              </Show>
            </div>
          </section>

          <section
            id="past-tournaments"
            class="min-h-[calc(100vh-4rem)] lg:snap-center flex flex-col items-center px-4 py-10 lg:p-12"
          >
            <div class="w-full max-w-7xl flex items-end gap-4 mb-8">
              <div class="flex-1">
                <h2 class="text-2xl font-bold">{t("tournament.past")}</h2>
                <p class="opacity-60 mt-1">{t("tournament.pastSubtitle")}</p>
              </div>
              <span class="icon-[fluent--history-20-regular] w-8 h-8 opacity-40" />
            </div>
            <div class="w-full max-w-7xl grid sm:grid-cols-2 xl:grid-cols-3 gap-6">
              <For each={history()} fallback={<p class="opacity-60">{t("tournament.emptyPast")}</p>}>
                {(item) => (
                  <Card class="overflow-hidden relative" contentClass="relative flex flex-col min-h-full">
                    <Picture
                      class="aspect-video"
                      src={item.cover ? mediaPath(item.cover) : bgTournamentDefault}
                      alt={item.name}
                    />
                    <Tag class="absolute top-2 right-2" level={lifecycleLevel(item.lifecycle)}>
                      <span>{t(`tournament.lifecycle.${item.lifecycle}`)}</span>
                    </Tag>
                    <button type="button" class="p-5 text-start flex-1" onClick={() => enterTournament(item)}>
                      <h3 class="text-lg font-bold">{item.name}</h3>
                      <p class="opacity-60 mt-2 line-clamp-2">{item.brief || t("tournament.noBrief")}</p>
                    </button>
                  </Card>
                )}
              </For>
            </div>
          </section>
        </div>
      </main>
    </>
  );
}
