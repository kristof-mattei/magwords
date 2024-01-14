// eslint-disable-next-line @typescript-eslint/triple-slash-reference
/// <reference types="vitest" />

import react from "@vitejs/plugin-react";
import { type UserConfig, defineConfig, loadEnv } from "vite";
import checker from "vite-plugin-checker";
import svgr from "vite-plugin-svgr";
import viteTsConfigPaths from "vite-tsconfig-paths";

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
    const env = loadEnv(mode, process.cwd());

    const config: UserConfig = {
        plugins: [
            react(),
            svgr(),
            viteTsConfigPaths(),
            checker({ typescript: true }),
        ],
        root: "front-end/src",
        build: {
            outDir: "../../dist",
            emptyOutDir: true,
            sourcemap: true,
            rollupOptions: {},
        },
        server: {
            port: parseInt(env["VITE_PORT"] ?? "") || 4000,
            host: true,
            strictPort: true,
            hmr: {
                host: "localhost",
                port: 4000,
            },
            // proxy: {
            //     "/api": {
            //         target: "http://localhost:3001",
            //         changeOrigin: true,
            //         secure: false,
            //         ws: true,
            //     },
            // },
        },
        resolve: {
            preserveSymlinks: true,
        },
        test: {
            globals: true,
            environment: "jsdom",
            environmentOptions: {
                jsdom: {},
            },
            outputFile: {},
            setupFiles: ["./test.setup.ts"],
            coverage: {
                provider: "v8",
                reportsDirectory: "../../coverage/vitest",
            },
        },
    };

    return config;
});
