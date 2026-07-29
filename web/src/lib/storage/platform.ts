import type { PlatformLicense } from "@models/platform";
import { makePersisted } from "@solid-primitives/storage";
import { createRoot } from "solid-js";
import { createStore, type StoreReturn } from "solid-js/store";

type PlatformStore = {
  version: string;
  accept_cookies: boolean;
  under_maintenance: boolean;
  backend_online: boolean;
  license: PlatformLicense | null;
  enable_ret2codec: boolean | null;
  readonly isOnline: boolean;
  readonly isCompatible: boolean;
};

export const frontendCompatVersion = import.meta.env.VITE_COMPAT_VERSION as string;

const platformRoot = createRoot(() =>
  makePersisted<PlatformStore, StoreReturn<PlatformStore>>(
    createStore<PlatformStore>({
      version: `${frontendCompatVersion}-UNKNOWN-0.0.0`,
      accept_cookies: false,
      under_maintenance: false,
      backend_online: false,
      license: null as null | PlatformLicense,
      enable_ret2codec: null as null | boolean,
      get isOnline() {
        return this.backend_online && !this.under_maintenance;
      },
      get isCompatible() {
        return this.version === frontendCompatVersion;
      },
    }),
    { name: "platform" }
  )
);

export const platformStore = platformRoot[0];
export const setPlatformStore = platformRoot[1];
