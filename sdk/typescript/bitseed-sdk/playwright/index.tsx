// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React from 'react'
import ReactDOM from 'react-dom'
import { Buffer } from 'buffer'

window.Buffer = Buffer
ReactDOM.render(<React.StrictMode></React.StrictMode>, document.getElementById('root'))
