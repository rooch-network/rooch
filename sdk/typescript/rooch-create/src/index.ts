#!/usr/bin/env node
// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { create } from 'create-create-app'
import { resolve } from 'path'
import packageJson from '../package.json'

const templateRoot = resolve(__dirname, '..', 'dist', 'templates')

// See https://github.com/uetchy/create-create-app/blob/master/README.md for other options.

create('create-rooch', {
  templateRoot,
  defaultTemplate: 'react',
  // templates use pnpm workspaces, so default to that for now
  // not sure if it's worth trying to support multiple kinds of package managers for monorepos, given the tooling is so different
  defaultPackageManager: 'pnpm',
  promptForDescription: false,
  promptForAuthor: false,
  promptForEmail: false,
  promptForLicense: false,
  promptForTemplate: true,
  caveat: ({ answers, packageManager }) =>
    `Done! Play in the rooch with \`cd ${answers.name}\` and \`${packageManager} run dev\``,
  extra: {
    'rooch-version': {
      type: 'input',
      describe: 'The version of Rooch packages to use, defaults to latest',
      default: packageJson.version,
    },
  },
})
