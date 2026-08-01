import nodePath from "node:path";

import { codecovVitePlugin } from "@codecov/vite-plugin";
import type { UserConfig } from "vite";
import { loadEnv } from "vite";
import { checker } from "vite-plugin-checker";
import svgr from "vite-plugin-svgr";
import type { ViteUserConfigFn } from "vitest/config";
import { coverageConfigDefaults, defineConfig } from "vitest/config";

function resolvePort(environmentValue: string | undefined, fallback: number): number {
    if (environmentValue === undefined || environmentValue === "") {
        return fallback;
    }

    const parsed = Number(environmentValue);

    return Number.isNaN(parsed) ? fallback : parsed;
}

const configFunction: ViteUserConfigFn = defineConfig(({ mode }) => {
    const environment = loadEnv(mode, process.cwd(), "");
    const port = resolvePort(environment["VITE_PORT"], 4000);

    const config: UserConfig = {
        appType: "spa",
        cacheDir: "../../node_modules/.cache/",
        css: {
            preprocessorOptions: {
                scss: {
                    silenceDeprecations: [
                        "color-functions",
                        "global-builtin",
                        "if-function",
                        "import",
                        "legacy-js-api",
                    ],
                },
            },
        },
        build: {
            minify: false,
            target: "esnext",
            emptyOutDir: true,
            sourcemap: true,
            outDir: "../../dist",
            rolldownOptions: {
                output: {
                    keepNames: true,
                },
            },
        },
        resolve: {
            alias: {
                "~bootstrap": nodePath.resolve(import.meta.dirname, "node_modules/bootstrap"),
            },
            tsconfigPaths: true,
        },
        plugins: [
            svgr(),
            checker({ typescript: true }),
            codecovVitePlugin({
                enableBundleAnalysis: environment["GITHUB_ACTIONS"] === "true",
                bundleName: "magwords-front-end",
                oidc: {
                    useGitHubOIDC: true,
                },
                telemetry: false,
            }),
        ],
        optimizeDeps: {
            noDiscovery: true,
            include: [],
        },
        root: "front-end/src",
        server: {
            port,
            host: true,
            strictPort: true,
            hmr: {
                host: "localhost",
                port,
            },
            cors: true,
            proxy: {
                "/ws": {
                    target: "http://localhost:3000",
                    changeOrigin: true,
                    secure: false,
                    ws: true,
                },
            },
        },
        test: {
            coverage: {
                exclude: [...coverageConfigDefaults.exclude, "./dependency-cruiser.config.mjs"],
                reporter: ["json", "html", "text"],
                provider: "v8",
                reportsDirectory: "../../coverage/vitest",
            },
            // environment: "jsdom",
            environmentOptions: {
                // jsdom: {},
            },
            globals: false,
            outputFile: {
                junit: "../../reports/vitest/test-report.xml",
            },
            restoreMocks: true,
            setupFiles: ["./test.setup.ts"],
        },
    };

    return config;
});

export default configFunction;
