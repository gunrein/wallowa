import * as fs from 'fs'

await fs.promises.cp('./static/', './dist/', { recursive: true })
