import type { Permission, Token } from "@models/user";
import { base64urlnopad } from "@scure/base";
import { makePersisted } from "@solid-primitives/storage";
import { createRoot } from "solid-js";
import { createStore, type StoreReturn } from "solid-js/store";

type AccountStore = {
  id: number | null;
  account: string | null;
  nickname: string | null;
  token: string | null;
  permissions: Permission[];
  warnedCodeGeneration: boolean;
};

const accountRoot = createRoot(() =>
  makePersisted<AccountStore, StoreReturn<AccountStore>>(
    createStore<AccountStore>({
      id: null as number | null,
      account: null as string | null,
      nickname: null as string | null,
      token: null as string | null,
      permissions: [] as Permission[],
      warnedCodeGeneration: false,
    }),
    { name: "account" }
  )
);

export const accountStore = accountRoot[0];
export const setAccountStore = accountRoot[1];

export function storeToken(token: string) {
  const parts = token?.split(".");
  if (!parts || parts.length < 3) return;
  const tokenRaw = new TextDecoder().decode(base64urlnopad.decode(parts[1]));
  const tokenJson = JSON.parse(tokenRaw) as Token;
  setAccountStore({
    id: tokenJson.id,
    account: tokenJson.account,
    nickname: tokenJson.nickname,
    token,
    permissions: tokenJson.permissions,
  });
}

export function resetUser() {
  setAccountStore({
    id: null,
    account: null,
    nickname: null,
    token: null,
    permissions: [],
    warnedCodeGeneration: false,
  });
}
