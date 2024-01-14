import type { Config } from "../lib/config";

type ImportMetaEnv = { readonly [P in keyof Config]: Config[P] };

interface ImportMeta {
    readonly env: ImportMetaEnv;
}
