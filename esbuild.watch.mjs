import * as esbuild from "esbuild";

let ctx = await esbuild.context({
  entryPoints: ["src-web/main.ts"],
  minify: false,
  bundle: true,
  sourcemap: true,
  target: ["es2020"],
  outdir: "dist",
});

await ctx.watch();
console.log("esbuild watching...");
