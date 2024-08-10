// Import styles, initialize component theme here.
// import '../src/common.css';

import React from 'react'
import ReactDOM from 'react-dom'
import { Buffer } from 'buffer';

window.Buffer = Buffer;
ReactDOM.render(<React.StrictMode></React.StrictMode>, document.getElementById('root'))
