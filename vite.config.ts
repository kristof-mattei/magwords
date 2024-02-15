import path from "path";

import { type UserConfig, loadEnv } from "vite";
import checker from "vite-plugin-checker";
import svgr from "vite-plugin-svgr";
import viteTsConfigPaths from "vite-tsconfig-paths";
import { defineConfig } from "vitest/config";

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
    const env = loadEnv(mode, process.cwd());

    const config: UserConfig = {
        plugins: [svgr(), viteTsConfigPaths(), checker({ typescript: true })],
        optimizeDeps: {
            exclude: ["src/entrypoints/index.ts"],
        },
        root: "front-end/src",
        build: {
            outDir: "../../dist",
            emptyOutDir: true,
            sourcemap: true,
            rollupOptions: {},
        },
        server: {
            port: parseInt(env.VITE_PORT ?? "") || 4000,
            host: true,
            strictPort: true,
            hmr: {
                host: "localhost",
                port: 4000,
            },
            cors: true,
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
            alias: {
                "~bootstrap": path.resolve(__dirname, "node_modules/bootstrap"),
            },
            preserveSymlinks: true,
        },
        test: {
            globals: true,
            // environment: "jsdom",
            environmentOptions: {
                // jsdom: {},
            },
            outputFile: {},
            setupFiles: ["./test.setup.ts"],
            coverage: {
                provider: "v8",
                reportsDirectory: "../../coverage/vitest",
            },
            cache: { dir: "../../node_modules/.vitest" },
        },
    };

    return config;
});
