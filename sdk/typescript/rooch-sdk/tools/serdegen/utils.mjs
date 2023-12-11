// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import fs from 'fs'
import os from 'os'
import path from 'path'
import { readFile, writeFile } from 'fs/promises'

export async function createTempFile(pattern) {
  return new Promise((resolve, reject) => {
    const tempDir = os.tmpdir()
    const filePath = path.join(tempDir, pattern)

    fs.writeFile(filePath, '', (err) => {
      if (err) {
        reject(err)
      } else {
        resolve(filePath)
      }
    })
  })
}

export function deleteTempFile(filePath) {
  return new Promise((resolve, reject) => {
    fs.unlink(filePath, (err) => {
      if (err) {
        reject(err)
      } else {
        resolve()
      }
    })
  })
}

export async function replaceFile(filePath, oldImport, newImport) {
  const fileContent = await readFile(filePath, 'utf8')
  const newContent = fileContent.replace(oldImport, newImport)
  await writeFile(filePath, newContent)
}
