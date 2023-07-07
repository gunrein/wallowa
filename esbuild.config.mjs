import * as esbuild from "esbuild";
import { glob } from 'glob';

const entryPoints = await glob('./src-web/**/*.ts');

console.log("esbuild building...");
await esbuild.build({
  entryPoints: entryPoints,
  minify: true,
  bundle: true,
  sourcemap: false,
  target: ["es2020"],
  outdir: "dist",
});
