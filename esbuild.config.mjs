import * as esbuild from "esbuild";

console.log("esbuild building...");
await esbuild.build({
  entryPoints: ["src-web/main.ts"],
  minify: true,
  bundle: true,
  sourcemap: false,
  target: ["es2020"],
  outdir: "dist",
});
