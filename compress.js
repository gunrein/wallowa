import { gzipSync, brotliCompressSync } from "zlib"
import * as fs from 'fs'
import { glob } from 'glob'

// Compress all the files in `./dist/`
const fileNames = await glob('./dist/**/*', { ignore: ["**/*.png"] })
for (const name of fileNames) {
  const stats = await fs.promises.stat(name)
  if (!stats.isDirectory()) {
    const data = fs.readFileSync(name);
    fs.writeFileSync(`${name}.gz`, gzipSync(data))
    fs.writeFileSync(`${name}.br`, brotliCompressSync(data))
  }
}
