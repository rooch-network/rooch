import { useCurrentAddress, useCurrentNetwork } from '@roochnetwork/rooch-sdk-kit';
import { Stack } from "@mui/material";
import { CopyToClipboard } from "react-copy-to-clipboard";
import { useMemo } from "react";
import { getTwitterShareText } from "../../utils/inviter";


export function ShareTwitter () {
  const address = useCurrentAddress()
  const network = useCurrentNetwork()
  const XText = useMemo( () => getTwitterShareText(network, address), [network, address])

  return (<CopyToClipboard
    text={XText}
    onCopy={() => {
      window.open(
        `https://twitter.com/intent/tweet?text=${encodeURIComponent(XText)}`,
        '_blank',
      )
    }}
  >
    <Stack
      className="font-medium cursor-pointer text-wrap bg-gray-200 p-3 rounded-md whitespace-pre-line">
      {XText}
    </Stack>
  </CopyToClipboard>)
}