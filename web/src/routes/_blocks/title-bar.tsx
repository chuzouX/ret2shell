import { useAccountProfile } from "@api/account";
import { getStaff, getTournament } from "@api/tournament";
import LogoAnimate from "@assets/animates/logo-animate";
import { mediaPath } from "@lib/utils/media";
import { strDisplayWidth } from "@lib/utils/string";
import type { Tournament } from "@models/tournament";
import { Permission } from "@models/user";
import { useLocation } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { t } from "@storage/theme";
import Card from "@widgets/card";
import Link from "@widgets/link";
import Popover from "@widgets/popover";
import clsx from "clsx";
import { createEffect, createMemo, createResource, createSignal, For, onCleanup, Show, untrack } from "solid-js";
import { Transition } from "solid-transition-group";
import I18nBox from "./i18n-box";
import NotificationBox from "./notification-box";
import ThemeBox from "./theme-box";
import UserBox from "./user-box";

function GlobalNav(props: { canAdmin: boolean; mobile?: boolean }) {
  const justify = () => (props.mobile ? "start" : undefined) as "start" | undefined;
  const textClass = () => (props.mobile ? "" : "hidden md:inline");
  return (
    <>
      <Link href="/tournaments" activeMatch="partial" ghost size="sm" justify={justify()}>
        <span class="icon-[fluent--trophy-20-regular] w-5 h-5" />
        <span class={textClass()}>{t("tournament.title")}</span>
      </Link>
      <Link href="/charts" activeMatch="exact" ghost size="sm" justify={justify()}>
        <span class="icon-[fluent--library-20-regular] w-5 h-5" />
        <span class={textClass()}>{t("tournament.charts.library")}</span>
      </Link>
      <Link href="/bulletin" activeMatch="partial" ghost size="sm" justify={justify()}>
        <span class="icon-[fluent--megaphone-20-regular] w-5 h-5" />
        <span class={textClass()}>{t("bulletin.title")}</span>
      </Link>
      <Show when={props.canAdmin}>
        <Link href="/admin" activeMatch="partial" ghost size="sm" justify={justify()}>
          <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
          <span class={textClass()}>{t("admin.title")}</span>
        </Link>
      </Show>
    </>
  );
}

function TournamentNav(props: { tournamentId: number; tournament?: Tournament; canManage: boolean; mobile?: boolean }) {
  const justify = () => (props.mobile ? "start" : undefined) as "start" | undefined;
  const textClass = () => (props.mobile ? "" : "hidden lg:inline");
  const links = () => {
    const items: Array<[string, string, string]> = [
      ["", "icon-[fluent--home-20-regular]", "tournament.nav.overview"],
      ["/charts", "icon-[fluent--music-note-2-20-regular]", "tournament.nav.charts"],
      ["/results", "icon-[fluent--document-checkmark-20-regular]", "tournament.nav.results"],
    ];
    if (props.tournament?.rules_visible) {
      items.splice(1, 0, ["/rules", "icon-[fluent--document-text-20-regular]", "tournament.nav.rules"]);
    }
    if (props.tournament?.announcements_visible) {
      items.splice(props.tournament?.rules_visible ? 2 : 1, 0, [
        "/announcements",
        "icon-[fluent--megaphone-loud-20-regular]",
        "tournament.nav.announcements",
      ]);
    }
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
            justify={justify()}
            title={t(label)}
          >
            <span class={`${iconClass} w-5 h-5 shrink-0`} />
            <span class={textClass()}>{t(label)}</span>
          </Link>
        )}
      </For>
      <Show when={props.canManage}>
        <Link
          href={`/tournaments/${props.tournamentId}/admin`}
          activeMatch="partial"
          ghost
          size="sm"
          justify={justify()}
          title={t("tournament.nav.admin")}
        >
          <span class="icon-[fluent--settings-20-regular] w-5 h-5 shrink-0" />
          <span class={textClass()}>{t("tournament.nav.admin")}</span>
        </Link>
      </Show>
      <Link
        href="/tournaments"
        ghost
        size="sm"
        level="warning"
        justify={justify()}
        title={t("general.actions.back.title")}
      >
        <span class="icon-[fluent--arrow-exit-20-regular] w-5 h-5 shrink-0" />
        <span class={textClass()}>{t("general.actions.back.title")}</span>
      </Link>
    </>
  );
}

export default function TitleBar() {
  const account = useAccountProfile({ enabled: () => !!accountStore.token });
  const location = useLocation();
  const tournamentId = createMemo(() => {
    const match = location.pathname.match(/^\/tournaments\/([^/]+)/);
    if (!match) return null;
    const id = Number.parseInt(match[1], 10);
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
  const tournamentName = () => {
    const id = tournamentId();
    const data = tournament();
    return id && data?.id === id ? data.name : "Rhythm Arena";
  };

  // Typing animation for title name
  const [typedName, setTypedName] = createSignal("");
  const [inClear, setInClear] = createSignal(false);
  let typeTimer: ReturnType<typeof setTimeout> | undefined;
  createEffect(() => {
    const name = tournamentName();
    if (name) {
      untrack(() => {
        clearTimeout(typeTimer);
        setInClear(true);
        typeTimer = setTimeout(() => {
          setTypedName(name);
          setInClear(false);
        }, 500);
      });
    }
  });
  onCleanup(() => clearTimeout(typeTimer));

  const hasCover = () => !!tournament()?.cover && !!tournamentId();

  return (
    <>
      <div id="page-top" class="print:hidden" />
      <header class="sticky top-0 z-50 w-full bg-layer/80 backdrop-blur-md border-b border-layer-content/15 print:hidden">
        <div class="h-16 px-3 max-w-screen-2xl mx-auto flex items-center gap-2">
          <Link
            href={tournamentId() ? `/tournaments/${tournamentId()}` : "/tournaments"}
            ghost
            class="font-bold shrink-0 flex items-center gap-1"
          >
            {/* Logo with flip transition */}
            <div class="w-6 h-6 shrink-0">
              <Transition
                name="fade-group-flip"
                mode="outin"
                onExit={(_el, done) => {
                  setTimeout(done, 300);
                }}
              >
                <Show when={hasCover()} fallback={<LogoAnimate class="w-6 h-6" />}>
                  <img
                    class="w-6 h-6 object-cover rounded"
                    src={mediaPath(tournament()!.cover)}
                    alt={tournament()?.name || ""}
                  />
                </Show>
              </Transition>
            </div>
            {/* Typing animation */}
            <div
              class={clsx(
                "transition-all duration-500 text-nowrap overflow-hidden border-r-2",
                inClear() ? "border-r-layer-content" : "border-r-transparent"
              )}
              style={{
                "max-width": inClear() ? "0px" : `${strDisplayWidth(typedName()) / 2 + 0.5}rem`,
              }}
            >
              {typedName()}
            </div>
          </Link>

          {/* Mobile navigation */}
          <div class="lg:hidden">
            <Popover
              square
              ghost
              popContentClass="pt-2"
              btnContent={<span class="icon-[fluent--navigation-20-regular] w-5 h-5" />}
            >
              <Card class="w-64" contentClass="p-2 flex flex-col gap-1">
                <Show when={tournamentId()} fallback={<GlobalNav canAdmin={!!canAdmin()} mobile />}>
                  <TournamentNav
                    tournamentId={tournamentId()!}
                    tournament={tournament()}
                    canManage={!!canManageTournament()}
                    mobile
                  />
                </Show>
              </Card>
            </Popover>
          </div>

          {/* Desktop navigation */}
          <nav class="hidden lg:flex items-center gap-1 overflow-x-auto">
            <Show when={tournamentId()} fallback={<GlobalNav canAdmin={!!canAdmin()} />}>
              <TournamentNav
                tournamentId={tournamentId()!}
                tournament={tournament()}
                canManage={!!canManageTournament()}
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
