import nodePath from "node:path";

import { codecovVitePlugin } from "@codecov/vite-plugin";
import type { UserConfig } from "vite";
import { loadEnv } from "vite";
import { checker } from "vite-plugin-checker";

import dts from "vite-plugin-dts";
import svgr from "vite-plugin-svgr";
import viteTsConfigPaths from "vite-tsconfig-paths";
import { coverageConfigDefaults, defineConfig } from "vitest/config";

export default defineConfig(({ mode }) => {
    const environment = loadEnv(mode, process.cwd());
    const port = Number.parseInt(environment["VITE_PORT"] ?? "");

    const config: UserConfig = {
        appType: "spa",
        css: {
            preprocessorOptions: {
                scss: {
                    silenceDeprecations: ["mixed-decls", "color-functions", "global-builtin", "import"],
                },
            },
        },
        build: {
            minify: false,
            target: "esnext",
            emptyOutDir: true,
            sourcemap: true,
            outDir: "../../dist",
            rollupOptions: {
                output: {},
            },
        },
        resolve: {
            alias: {
                "@/": nodePath.resolve("src/"),
                "~bootstrap": nodePath.resolve(import.meta.dirname, "node_modules/bootstrap"),
            },
        },
        plugins: [
            svgr(),
            viteTsConfigPaths(),
            dts(),
            checker({ typescript: true }),
            codecovVitePlugin({
                enableBundleAnalysis: environment["CODECOV_TOKEN"] !== undefined,
                bundleName: "library",
                uploadToken: environment["CODECOV_TOKEN"] ?? "",
            }),
        ],
        optimizeDeps: {
            noDiscovery: true,
            // exclude: ["src/entrypoints/index.ts"],
        },
        root: "front-end/src",
        server: {
            port: Number.isNaN(port) ? 4000 : port,
            host: true,
            strictPort: true,
            hmr: {
                host: "localhost",
                port: 4000,
            },
            cors: true,
            proxy: {
                "/socket.io": {
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
