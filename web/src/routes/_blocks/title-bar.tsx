import { useAccountProfile } from "@api/account";
import { getStaff, getTournament } from "@api/tournament";
import type { Tournament } from "@models/tournament";
import { Permission } from "@models/user";
import { useLocation, useParams } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { t } from "@storage/theme";
import Link from "@widgets/link";
import { createMemo, createResource, For, Show } from "solid-js";
import I18nBox from "./i18n-box";
import NotificationBox from "./notification-box";
import ThemeBox from "./theme-box";
import UserBox from "./user-box";

function GlobalNav(props: { canAdmin: boolean }) {
  return (
    <>
      <Link href="/tournaments" activeMatch="partial" ghost size="sm">
        <span class="icon-[fluent--trophy-20-regular] w-5 h-5" />
        <span class="hidden md:inline">{t("tournament.title")}</span>
      </Link>
      <Link href="/wiki" activeMatch="partial" ghost size="sm">
        <span class="icon-[fluent--book-number-20-regular] w-5 h-5" />
        <span class="hidden md:inline">{t("wiki.title")}</span>
      </Link>
      <Link href="/bulletin" activeMatch="partial" ghost size="sm">
        <span class="icon-[fluent--megaphone-20-regular] w-5 h-5" />
        <span class="hidden md:inline">{t("bulletin.title")}</span>
      </Link>
      <Show when={props.canAdmin}>
        <Link href="/admin" activeMatch="partial" ghost size="sm">
          <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
          <span class="hidden md:inline">{t("admin.title")}</span>
        </Link>
      </Show>
    </>
  );
}

function TournamentNav(props: { tournamentId: number; tournament?: Tournament; canManage: boolean }) {
  const links = () => {
    const items: Array<[string, string, string, string]> = [
      ["", "icon-[fluent--home-20-regular]", "tournament.nav.overview"],
      ["/charts", "icon-[fluent--music-note-2-20-regular]", "tournament.nav.charts"],
      ["/results", "icon-[fluent--document-checkmark-20-regular]", "tournament.nav.results"],
    ];
    if (props.tournament?.competition_mode !== "team") {
      items.push(["/leaderboard/individual", "icon-[fluent--person-star-20-regular]", "tournament.nav.individual"]);
    }
    if (props.tournament?.competition_mode !== "individual") {
      items.push(["/leaderboard/team", "icon-[fluent--people-team-20-regular]", "tournament.nav.team"]);
    }
    return items;
  };
  return (
    <>
      <For each={links()}>
        {([path, iconClass, label]) => (
          <Link
            href={`/tournaments/${props.tournamentId}${path}`}
            activeMatch={path ? "partial" : "exact"}
            ghost
            size="sm"
            title={t(label)}
          >
            <span class={`${iconClass} w-5 h-5 shrink-0`} />
            <span class="hidden lg:inline">{t(label)}</span>
          </Link>
        )}
      </For>
      <Show when={props.canManage}>
        <Link
          href={`/tournaments/${props.tournamentId}/admin`}
          activeMatch="partial"
          ghost
          size="sm"
          title={t("tournament.nav.admin")}
        >
          <span class="icon-[fluent--settings-20-regular] w-5 h-5 shrink-0" />
          <span class="hidden lg:inline">{t("tournament.nav.admin")}</span>
        </Link>
      </Show>
      <Link href="/tournaments" ghost size="sm" level="warning" title={t("general.actions.back.title")}>
        <span class="icon-[fluent--arrow-exit-20-regular] w-5 h-5 shrink-0" />
        <span class="hidden lg:inline">{t("general.actions.back.title")}</span>
      </Link>
    </>
  );
}

export default function TitleBar() {
  const account = useAccountProfile({ enabled: () => !!accountStore.token });
  const location = useLocation();
  const params = useParams();
  const tournamentId = createMemo(() => {
    if (!location.pathname.startsWith("/tournaments/")) return null;
    const id = Number.parseInt(params.tournament || "", 10);
    return Number.isFinite(id) ? id : null;
  });
  const [tournament] = createResource(tournamentId, getTournament);
  const [staff] = createResource(() => (accountStore.id && tournamentId() ? tournamentId() : null), getStaff);
  const canAdmin = () =>
    account.data?.permissions.some((permission) =>
      [Permission.Statistics, Permission.DevOps, Permission.User].includes(permission)
    );
  const canManageTournament = () =>
    tournament()?.owner_id === accountStore.id || staff()?.some((member) => member.user_id === accountStore.id);

  return (
    <>
      <div id="page-top" class="print:hidden" />
      <header class="sticky top-0 z-50 w-full bg-layer/80 backdrop-blur-md border-b border-layer-content/15 print:hidden">
        <div class="h-16 px-3 max-w-screen-2xl mx-auto flex items-center gap-2">
          <Link
            href={tournamentId() ? `/tournaments/${tournamentId()}` : "/tournaments"}
            ghost
            class="font-bold shrink-0"
          >
            <span class="icon-[fluent--music-note-2-20-filled] w-6 h-6 text-primary shrink-0" />
            <span class="hidden sm:inline max-w-48 truncate">{tournament()?.name || "Rhythm Arena"}</span>
          </Link>
          <nav class="flex items-center gap-1 overflow-x-auto">
            <Show when={tournamentId()} fallback={<GlobalNav canAdmin={!!canAdmin()} />}>
              <TournamentNav
                tournamentId={tournamentId()!}
                tournament={tournament()}
                canManage={canManageTournament()}
              />
            </Show>
          </nav>
          <span class="flex-1" />
          <div class="hidden sm:flex items-center gap-1">
            <NotificationBox />
            <ThemeBox />
            <I18nBox />
          </div>
          <UserBox />
        </div>
      </header>
    </>
  );
}
