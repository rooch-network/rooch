// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import fs from 'fs'
import path from 'path'
import { readFile } from 'fs/promises'

export class Optimize {
  constructor(runtimePath) {
    this.runtimePath = runtimePath
  }

  // Helper method to actually modify a file's imports
  updateFileImports(filePath) {
    // Read the file content
    let content = fs.readFileSync(filePath, 'utf8')

    const importRegex = /import\s+\{?(.*?)\}?\s+from\s+['"](.*)\.ts['"];?/g
    content = content.replace(importRegex, (match, p1, p2) => `import {${p1}} from '${p2}'`)

    // Save the modified content back to the file
    fs.writeFileSync(filePath, content)
  }

  //export * from "./bcsDeserializer.ts" to export * from './bcsDeserializer
  updateFileExports(filePath) {
    // Read the file content
    let content = fs.readFileSync(filePath, 'utf8')
    const importRegex = /export\s+(.*?)\s+from\s+['"](.*)\.ts['"];?/g
    content = content.replace(importRegex, (match, p1, p2) => `export ${p1} from '${p2}'`)

    // Save the modified content back to the file
    fs.writeFileSync(filePath, content)
  }

  // Method to optimize imports in runtime directory
  async optimizeRuntimeImports() {
    const readFilesRecursively = (dir) => {
      let results = []
      const list = fs.readdirSync(dir)

      list.forEach((file) => {
        file = dir + '/' + file
        const stat = fs.statSync(file)
        if (stat && stat.isDirectory()) {
          /* Recurse into a subdirectory */
          results = results.concat(readFilesRecursively(file))
        } else {
          /* Is a file */
          results.push(file)
        }
      })

      return results.filter((file) => file.endsWith('.ts')) // Process only JS files
    }

    try {
      const files = readFilesRecursively(this.runtimePath)

      files.forEach((file) => {
        this.updateFileImports(file)
        this.updateFileExports(file)
      })

      console.log('Optimization of runtime imports completed.')
    } catch (error) {
      console.error('An error occurred during the optimization process:', error)
    }
  }

  // Method to remove unused imports as specified by a config JSON
  async cleanRoochTypesImports(configJsonPath) {
    const configContent = await readFile(configJsonPath, 'utf8')
    const config = JSON.parse(configContent)
    const { unusedImports } = config
    const modFilePath = path.join(this.runtimePath, 'rooch_types', 'mod.ts')
    let content = fs.readFileSync(modFilePath, 'utf8')

    const handledUnusedImports = new Set()

    const importStatementRegex = /(?<=import\s+{)([^}]*)(?=\}\s+from\s+['"][^'"]+['"];?)/gm

    const replaceUnusedImports = (match) => {
      return match
        .split(',')
        .map((importName) => importName.trim())
        .filter((importName) => {
          if (!unusedImports.includes(importName) || handledUnusedImports.has(importName)) {
            return true
          }
          handledUnusedImports.add(importName)
          return false
        })
        .join(', ')
    }

    content = content.replace(importStatementRegex, replaceUnusedImports)

    content = content.replace(/import\s+{\s*}\s+from\s+['"][^'"]+['"];?(\r?\n|\r)/g, '')
    content = content.replace(/,\s*}/g, '}')
    content = content.replace(/{\s*,/g, '{')

    fs.writeFileSync(modFilePath, content.trim())
  }
}
