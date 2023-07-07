import * as esbuild from 'esbuild';
import { glob } from 'glob';

const entryPoints = await glob('./src-web/**/*.ts');

let ctx = await esbuild.context({
  entryPoints: entryPoints,
  minify: false,
  bundle: true,
  sourcemap: true,
  target: ["es2020"],
  outdir: "dist",
});

await ctx.watch();
console.log("esbuild watching...");
