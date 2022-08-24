import rust from "@wasm-tool/rollup-plugin-rust";
import serve from "rollup-plugin-serve";
import livereload from "rollup-plugin-livereload";
import { terser } from "rollup-plugin-terser";

const is_watch = !!process.env.ROLLUP_WATCH;

// Needed to stop the browser from complaining about import.meta
function fixImportMeta() {
    return {
        name: "fix import meta",

        resolveImportMeta(property, info) {
            if (property === "url") {
                return "IMPORT_META";
            }

            return null;
        },
    };
}

export default {
    input: {
        index: "./Cargo.toml",
        thread1: "./src/thread1/Cargo.toml",
        thread2: "./src/thread2/Cargo.toml",
    },
    output: {
        dir: "dist/js",
        format: "es",
        sourcemap: true,
    },
    plugins: [
        rust({
            serverPath: "/js/",
        }),

        is_watch && serve({
            contentBase: "dist",
            open: true,
        }),

        is_watch && livereload("dist"),

        !is_watch && terser(),

        fixImportMeta(),
    ],
};
