import { handleHttpError } from "@api";
import { deleteTournament, getTournaments, updateTournament } from "@api/tournament";
import type { TournamentLifecycle } from "@models/tournament";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Dialog from "@widgets/dialog";
import Link from "@widgets/link";
import { createResource, createSignal, For, Show } from "solid-js";

const lifecycleNext: Record<string, TournamentLifecycle | undefined> = {
  draft: "registration",
  registration: "running",
  running: "review",
  review: "finished",
  finished: "archived",
};

function lifecycleColor(l: TournamentLifecycle) {
  if (l === "registration") return "text-info";
  if (l === "running") return "text-success";
  if (l === "review" || l === "finished") return "text-warning";
  if (l === "archived") return "text-error";
  return "opacity-50";
}

export default function () {
  const [tournaments, { refetch }] = createResource(getTournaments);
  const [busy, setBusy] = createSignal(false);
  const [confirmOpen, setConfirmOpen] = createSignal(false);
  const [confirmMsg, setConfirmMsg] = createSignal("");
  const [confirmAction, setConfirmAction] = createSignal<() => Promise<unknown>>(() => Promise.resolve());

  const run = async (action: () => Promise<unknown>) => {
    setBusy(true);
    try {
      await action();
      await refetch();
    } catch (error) {
      handleHttpError(error as Error, t("tournament.errors.action"));
    } finally {
      setBusy(false);
    }
  };

  const askConfirm = (msg: string, action: () => Promise<unknown>) => {
    setConfirmMsg(msg);
    setConfirmAction(() => action);
    setConfirmOpen(true);
  };

  return (
    <main class="flex-1 p-4 lg:p-8 max-w-7xl mx-auto w-full space-y-6">
      <div>
        <h1 class="text-2xl font-bold">比赛管理</h1>
        <p class="opacity-60 mt-1">查看、编辑、删除所有比赛。</p>
      </div>

      <div class="border border-layer-content/15 rounded-lg overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="bg-layer-content/5 text-left border-b border-layer-content/15">
              <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider">ID</th>
              <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider">名称</th>
              <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider whitespace-nowrap">状态</th>
              <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider whitespace-nowrap">模式</th>
              <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider whitespace-nowrap hidden md:table-cell">
                创建者
              </th>
              <th class="p-3 font-bold text-xs opacity-50 uppercase tracking-wider text-right">操作</th>
            </tr>
          </thead>
          <tbody>
            <For each={tournaments()}>
              {(item) => (
                <tr class="border-b border-layer-content/10 hover:bg-layer-content/5 transition-colors">
                  <td class="p-3 font-mono opacity-40 text-xs">#{item.id}</td>
                  <td class="p-3">
                    <Link href={`/tournaments/${item.id}`} class="font-bold truncate max-w-48 block" ghost size="sm">
                      {item.name}
                    </Link>
                  </td>
                  <td class="p-3 whitespace-nowrap">
                    <span
                      class="inline-flex items-center gap-1.5 text-xs font-medium"
                      classList={{ [lifecycleColor(item.lifecycle)]: true }}
                    >
                      <span
                        class="w-1.5 h-1.5 rounded-full"
                        classList={{
                          "bg-info": item.lifecycle === "registration",
                          "bg-success": item.lifecycle === "running",
                          "bg-warning": item.lifecycle === "review" || item.lifecycle === "finished",
                          "bg-error": item.lifecycle === "archived",
                          "bg-layer-content/30": item.lifecycle === "draft",
                        }}
                      />
                      {t(`tournament.lifecycle.${item.lifecycle}`)}
                    </span>
                  </td>
                  <td class="p-3 opacity-60 text-xs whitespace-nowrap">
                    {t(`tournament.mode.${item.competition_mode}`)}
                  </td>
                  <td class="p-3 opacity-40 text-xs hidden md:table-cell">#{item.owner_id}</td>
                  <td class="p-3">
                    <div class="flex items-center justify-end gap-1">
                      <Show when={lifecycleNext[item.lifecycle]}>
                        <Button
                          size="sm"
                          ghost
                          disabled={busy()}
                          onClick={() =>
                            run(
                              async () => await updateTournament(item.id, { lifecycle: lifecycleNext[item.lifecycle]! })
                            )
                          }
                        >
                          <span class="icon-[fluent--arrow-next-20-regular] w-4 h-4" />
                        </Button>
                      </Show>
                      <Link href={`/tournaments/${item.id}/admin`} size="sm" ghost>
                        <span class="icon-[fluent--settings-20-regular] w-4 h-4" />
                      </Link>
                      <Button
                        size="sm"
                        ghost
                        level="error"
                        disabled={busy()}
                        onClick={() =>
                          askConfirm(`确定删除比赛 "${item.name}"？此操作不可撤销！`, async () => {
                            await deleteTournament(item.id);
                          })
                        }
                      >
                        <span class="icon-[fluent--delete-20-regular] w-4 h-4" />
                      </Button>
                    </div>
                  </td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>

      <Dialog
        open={confirmOpen()}
        onOpenChange={(e) => setConfirmOpen(e.open)}
        btnContent=""
        modal
        level="error"
        class="hidden"
      >
        <div class="space-y-4 min-w-[280px]">
          <p class="text-sm">{confirmMsg()}</p>
          <div class="flex justify-end gap-2">
            <Button size="sm" ghost onClick={() => setConfirmOpen(false)}>
              {t("general.actions.cancel.title")}
            </Button>
            <Button
              size="sm"
              level="error"
              onClick={() =>
                run(async () => {
                  await confirmAction()();
                  setConfirmOpen(false);
                })
              }
            >
              {t("general.actions.delete.title")}
            </Button>
          </div>
        </div>
      </Dialog>
    </main>
  );
}
