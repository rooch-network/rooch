import type { DialogProps } from '@mui/material';

import { useState, useEffect } from 'react';

import {
  Box,
  List,
  Stack,
  Dialog,
  darken,
  styled,
  Divider,
  ListItem,
  TextField,
  Typography,
  ListItemIcon,
  ListItemText,
  DialogContent,
  InputAdornment,
  ListItemButton,
  ListItemSecondaryAction,
} from '@mui/material';

import { formatCoin, toBigNumber } from 'src/utils/number';

import { grey } from 'src/theme/core';

import Text from './typography/text';
import { Iconify } from '../iconify';

import type { UserCoin } from './types';

export interface CoinSelectButtonProps {
  coins: UserCoin[];
  coin?: UserCoin;
  disabledCoins: string[];
  fixedSwap?: boolean;
  onSelect: (coin: UserCoin) => void;
}

export default function SwapCoinSelectButton({
  coins,
  coin,
  disabledCoins,
  fixedSwap,
  onSelect,
}: CoinSelectButtonProps) {
  const [showModal, setShowModal] = useState(false);
  const [search, setSearch] = useState('');
  const [filterCoins, setFilterCoins] = useState<UserCoin[]>(coins);

  useEffect(() => {
    if (search) {
      setFilterCoins(coins.filter((it) => it.name.toLowerCase().includes(search.toLowerCase())));
    } else {
      setFilterCoins(coins);
    }
  }, [search, coins]);

  return (
    <Stack>
      {fixedSwap ? (
        <Stack
          direction="row"
          spacing={1}
          justifyContent="flex-end"
          sx={{
            width: '160px',
            height: '48px',
            padding: '12px',
            borderRadius: '6px',
          }}
        >
          {coin && <Box component="img" src={coin.icon} width={24} />}
          <Text>{coin ? coin.symbol : 'Select Token'}</Text>
        </Stack>
      ) : (
        <Stack
          direction="row"
          spacing={1}
          onClick={() => {
            setShowModal(true);
          }}
          sx={{
            width: '160px',
            height: '48px',
            padding: '12px',
            borderRadius: '6px',
            border: '1px solid #D0D5DD',
            background: '#FFF',
            boxShadow: '0px 1px 2px 0px rgba(16, 24, 40, 0.05)',
            cursor: 'pointer',
            '&:hover': {
              background: darken('#FFF', 0.025),
            },
          }}
        >
          {coin && <Box component="img" src={coin.icon} width={24} />}
          <Text sx={{ flexGrow: 1 }}>{coin ? coin.symbol : 'Select Token'}</Text>
          <Box component="img" src="assets/icons/swap/chevron-down.svg" />
        </Stack>
      )}

      <CoinSelectDialog
        open={showModal}
        onClose={() => {
          setShowModal(false);
        }}
        maxWidth="md"
      >
        <DialogContent>
          <TextField
            className="search"
            fullWidth
            placeholder="Search Token"
            sx={{ margin: '8px 0' }}
            InputProps={{
              startAdornment: (
                <InputAdornment position="start">
                  <Iconify icon="solar:magnifer-outline" />
                </InputAdornment>
              ),
              endAdornment: (
                <InputAdornment
                  position="end"
                  onClick={() => {
                    setShowModal(false);
                  }}
                  sx={{
                    cursor: 'pointer',
                  }}
                >
                  <div className="esc-button">Esc</div>
                </InputAdornment>
              ),
            }}
            onChange={(e) => {
              if (e.target.value.toLowerCase() === 'usdc') {
                setSearch('usd coin');
              } else if (e.target.value.toLowerCase() === 'usdt') {
                setSearch('tether usd');
              } else {
                setSearch(e.target.value);
              }
            }}
          />
          <Divider />
          <List
            sx={{
              p: '8px',
            }}
          >
            {filterCoins.map((coin, index) => (
              <ListItem
                key={index}
                disablePadding
                onClick={() => {
                  if (!disabledCoins.includes(coin.coinType)) {
                    onSelect(coin);
                    setSearch('');
                    setShowModal(false);
                  }
                }}
              >
                <ListItemButton
                  disabled={disabledCoins.includes(coin.coinType)}
                  sx={{
                    borderRadius: '4px',
                  }}
                >
                  <ListItemIcon>
                    <Box component="img" src={coin.icon} alt={coin.name} />
                  </ListItemIcon>
                  <ListItemText>
                    <Typography className="name">{coin.symbol}</Typography>
                    <Typography className="desc">{coin.name}</Typography>
                  </ListItemText>
                  {toBigNumber(coin.balance).gt(0) && (
                    <ListItemSecondaryAction>
                      <Typography className="amount">{formatCoin(coin, true)}</Typography>
                    </ListItemSecondaryAction>
                  )}
                </ListItemButton>
              </ListItem>
            ))}
          </List>
        </DialogContent>
      </CoinSelectDialog>
    </Stack>
  );
}

export const CoinSelectDialog = styled(Dialog)<DialogProps>(() => ({
  img: {
    width: '24px',
    height: '24px',
  },
  '& .MuiDialog-paper': {
    width: '420px',
  },
  '& .MuiDialogContent-root': {
    padding: 0,
  },
  '& .MuiListItemButton-root': {
    padding: '12px 20px',
  },
  '& .MuiListItemIcon-root': {
    minWidth: '24px',
    marginRight: '12px',
  },
  '& .search fieldset': {
    border: 'none',
  },
  '& .name': {
    fontWeight: 500,
    fontSize: '1rem',
    lineHeight: '24px',
    color: grey[900],
  },
  '& .desc': {
    fontWeight: 500,
    fontSize: '0.75rem',
    lineHeight: '14px',
    color: 'rgba(0, 0, 0, 0.35)',
  },
  '& .amount': {
    fontWeight: 500,
    fontSize: '0.75rem',
    lineHeight: '14px',
    color: 'rgba(0, 0, 0, 0.35)',
  },
  '& .esc-button': {
    padding: '3px 8px',
    background: '#fff',
    border: `1px solid ${grey[300]}`,
    borderRadius: '6px',
    fontWeight: 500,
    fontSize: '0.75rem',
    lineHeight: '18px',
    color: grey[700],
  },
}));
