// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// ** Next Imports
import dynamic from 'next/dynamic'

// ! To avoid 'Window is not defined' error
const ReactApexcharts = dynamic(() => import('react-apexcharts'), { ssr: false })

export default ReactApexcharts
