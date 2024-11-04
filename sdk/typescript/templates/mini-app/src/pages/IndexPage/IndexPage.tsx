import {Section, Cell, Image, List} from '@telegram-apps/telegram-ui';
import {FC, useEffect, useState} from 'react';

import { Link } from '@/components/Link/Link.tsx';
import { Page } from '@/components/Page.tsx';

import tonSvg from './ton.svg';
import { useTonConnectUI, useTonWallet } from "@tonconnect/ui-react";
import {ConnectRoochPage} from "@/pages/ConnectRoochPage/ConnectRoochPage.tsx";

const LOCA_LSTORAG_EKEY = 'session-key'

export const IndexPage: FC = () => {
  const [showAuth, setShowAuth] = useState(false);
  const wallet = useTonWallet();
  const [tonConnectUI] = useTonConnectUI();
  const localSessionKey = localStorage.getItem(LOCA_LSTORAG_EKEY);
  useEffect(() => {
      if (localSessionKey !== null) {
          // TODO: check
          return
      }
      console.log(wallet, localSessionKey)
      if (wallet === null || localSessionKey === null) {
          setShowAuth(true)
      }
  }, [localSessionKey])

  return (
    <Page back={false}>
      <List>
        <Section header="Rooch Clicker" footer="Join our Click Challenge!">
          <Cell
            before={<Image src={tonSvg} style={{ backgroundColor: '#007AFF' }} />}
            subtitle="Connect your TON wallet"
            onClick={() => {
              if (wallet?.account) {
                return
              }
              tonConnectUI.openModal()
              setShowAuth(true)
            }}
          >
            {wallet?.account ? wallet?.account.address : 'TON Connect'}
          </Cell>
        </Section>
        {showAuth ? (
          <ConnectRoochPage onConnectSuccess={() => setShowAuth(false)}/>
        ) : (
          <Section
            header="Application Launch Data"
            footer="These pages help developer to learn more about current launch information"
          >
            <Link to="/init-data">
              <Cell subtitle="User data, chat information, technical data">Init Data</Cell>
            </Link>
            <Link to="/launch-params">
              <Cell subtitle="Platform identifier, Mini Apps version, etc.">Launch Parameters</Cell>
            </Link>
            <Link to="/rooch">
              <Cell subtitle="Test">Rooch</Cell>
            </Link>
            <Link to="/theme-params">
              <Cell subtitle="Telegram application palette information">Theme Parameters</Cell>
            </Link>
          </Section>
        )}
      </List>
    </Page>
  )
};