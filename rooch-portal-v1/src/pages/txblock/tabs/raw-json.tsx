// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism'
import { TransactionWithInfoView } from '@roochnetwork/rooch-sdk'

type RawJsonProps = {
  txData: TransactionWithInfoView
}

// TODO: Improve ui
export const RawJson: React.FC<RawJsonProps> = ({ txData }) => {
  return (
    <div className="p-0 dark:bg-inherit">
      <div className="flex flex-col items-start justify-start gap-3">
        <div className="flex flex-col items-start justify-start gap-5 font-medium">
          <div className="rounded-lg flex flex-col items-start">
            <SyntaxHighlighter
              language="json"
              style={vscDarkPlus}
              customStyle={{
                whiteSpace: 'pre-wrap',
                overflow: 'scroll',
                width: '100%',
                borderRadius: '9px',
              }}
              wrapLongLines={true}
            >
              {JSON.stringify(txData, null, 2)}
            </SyntaxHighlighter>
          </div>
        </div>
      </div>
    </div>
  )
}
