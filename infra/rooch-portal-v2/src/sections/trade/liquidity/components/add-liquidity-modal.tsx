import type { BalanceInfoView, AnnotatedMoveStructView } from '@roochnetwork/rooch-sdk';

import BigNumber from 'bignumber.js';
import { useDebounce } from 'react-use';
import { Args, Transaction } from '@roochnetwork/rooch-sdk';
import { useMemo, useState, useEffect, useCallback } from 'react';
import {
  SessionKeyGuard,
  useCurrentAddress,
  useRoochClientQuery,
  useSignAndExecuteTransaction,
} from '@roochnetwork/rooch-sdk-kit';

import { LoadingButton } from '@mui/lab';
import {
  Box,
  Stack,
  Button,
  Dialog,
  Divider,
  TextField,
  Typography,
  DialogTitle,
  FormControl,
  DialogActions,
  DialogContent,
  InputAdornment,
} from '@mui/material';

import { useRouter } from 'src/routes/hooks';

import { useNetworkVariable } from 'src/hooks/use-networks';

import { formatCoin } from 'src/utils/format-number';
import { toDust, fromDust, formatByIntl, bigNumberToBigInt } from 'src/utils/number';

import { toast } from 'src/components/snackbar';

import Icon from './icon';

import type { AllLiquidityItemType } from '../../hooks/use-all-liquidity';

const STEPS = ['Deposit amounts', 'You will receive'];

export default function AddLiquidityModal({
  open,
  onClose,
  row,
}: {
  open: boolean;
  onClose: () => void;
  row: AllLiquidityItemType;
}) {
  const dex = useNetworkVariable('dex');
  const currentAddress = useCurrentAddress();
  const [xAmount, setXAmount] = useState('');
  const [yAmount, setYAmount] = useState('');
  const { mutateAsync, isPending } = useSignAndExecuteTransaction();
  const [slippage, setSlippage] = useState(0.005);
  const [customSlippage, setCustomSlippage] = useState('');
  const [activeStep, setActiveStep] = useState(0);
  const [yLabelError, setYLabelError] = useState<string>();
  const router = useRouter();

  const { data: lpTotalSupply } = useRoochClientQuery('queryObjectStates', {
    filter: {
      object_id: row.lpTokenId,
    },
  });

  const { data: reserveX } = useRoochClientQuery('queryObjectStates', {
    filter: {
      object_id: row.x.id,
    },
  });

  const { data: reserveY } = useRoochClientQuery('queryObjectStates', {
    filter: {
      object_id: row.y.id,
    },
  });

  const { data: xCoinInfo } = useRoochClientQuery('getBalance', {
    coinType: row.x.type,
    owner: currentAddress?.toStr() || ''
  });

  const { data: yCoinInfo } = useRoochClientQuery('getBalance', {
    coinType: row.y.type,
    owner: currentAddress?.toStr() || ''
  });

  const receive = useMemo(() => {
    if (
      activeStep === 0 ||
      !lpTotalSupply ||
      !reserveX ||
      !reserveY ||
      xAmount === '' ||
      yAmount === '' ||
      !xCoinInfo ||
      !yCoinInfo
    ) {
      return {
        liquidity: '-',
        share: '-',
      };
    }

    const lpView = lpTotalSupply.data[0].decoded_value!.value;
    const totalSupply = lpView.supply as string;
    const lpDecimals = lpView.decimals as number;
    const xBalance = (reserveX.data[0].decoded_value!.value.balance as AnnotatedMoveStructView)
      .value.value as string;
    const yBalance = (reserveY.data[0].decoded_value!.value.balance as AnnotatedMoveStructView)
      .value.value as string;

    const fixedXAmount = toDust(xAmount, xCoinInfo.decimals || 0);
    const fixedYAmount = toDust(yAmount, yCoinInfo.decimals || 0);

    const liquidity = Math.min(
      BigNumber(fixedXAmount.toString()).multipliedBy(totalSupply).div(xBalance).toNumber(),
      BigNumber(fixedYAmount.toString()).multipliedBy(totalSupply).div(yBalance).toNumber()
    );
    const share = BigNumber(liquidity.toString()).div(totalSupply).toFixed(4, 1);
    return { liquidity: formatCoin(liquidity, lpDecimals, 2), share };
  }, [lpTotalSupply, reserveX, reserveY, xAmount, yAmount, xCoinInfo, yCoinInfo, activeStep]);

  const handleNext = () => {
    setActiveStep(activeStep + 1);
  };

  const handleBack = () => {
    setActiveStep(activeStep - 1);
  };

  useEffect(() => {
    if (!currentAddress) {
      onClose();
    }
  }, [currentAddress, onClose]);

  const handleAddLiquidity = () => {
    const fixedX = toDust(xAmount.replaceAll(',', ''), xCoinInfo!.decimals || 0);
    const fixedY = toDust(yAmount.replaceAll(',', ''), yCoinInfo!.decimals || 0);
    const finalSlippage =
      slippage === 0
        ? customSlippage === '' || customSlippage === '0'
          ? 0
          : Number(customSlippage)
        : slippage;
    const minX = finalSlippage
      ? bigNumberToBigInt(
          BigNumber(fixedX.toString()).minus(
            BigNumber(fixedX.toString()).multipliedBy(BigNumber(slippage))
          )
        )
      : fixedX;
    const minY = finalSlippage
      ? bigNumberToBigInt(
          BigNumber(fixedY.toString()).minus(
            BigNumber(fixedY.toString()).multipliedBy(BigNumber(slippage))
          )
        )
      : fixedY;
    const tx = new Transaction();
    tx.callFunction({
      target: `${dex.address}::router::add_liquidity`,
      args: [Args.u64(fixedX), Args.u64(fixedY), Args.u64(minX), Args.u64(minY)],
      typeArgs: [row.x.type, row.y.type],
    });
    mutateAsync({
      transaction: tx,
    })
      .then((result) => {
        if (result.execution_info.status.type === 'executed') {
          toast.success('add success');
        } else {
          toast.error('add failed');
        }
      })
      .catch((e: any) => {
        console.log(e);
        toast.error('add failed');
      })
      .finally(() => {
        onClose();
      });
  };

  const fetchY = useCallback(() => {
    if (xAmount === '' || xAmount === '0' || !reserveX || !reserveY || !xCoinInfo || !yCoinInfo) {
      return;
    }

    console.log('hahah')
    const xBalance = (reserveX.data[0].decoded_value!.value.balance as AnnotatedMoveStructView)
      .value.value as string;
    const yBalance = (reserveY.data[0].decoded_value!.value.balance as AnnotatedMoveStructView)
      .value.value as string;
    const fixedX = toDust(xAmount.replaceAll(',', ''), xCoinInfo.decimals || 0);
    const xRate = BigNumber(fixedX.toString()).div(xBalance);
    const y = BigNumber(yBalance).multipliedBy(xRate);

    if (y.toNumber() > Number(yCoinInfo.balance || 0)) {
      setYLabelError('Insufficient');
    } else {
      setYLabelError(undefined);
    }
    setYAmount(formatByIntl(fromDust(y.toFixed(0, 1), yCoinInfo.decimals || 0).toString()));
  }, [xAmount, reserveX, reserveY, xCoinInfo, yCoinInfo]);

  useDebounce(fetchY, 500, [fetchY]);

  const getStepContent = (step: number) => {
    switch (step) {
      case 0:
        return (
          <>
            <Stack
              direction="row"
              className="mb-2 w-full"
              justifyContent="space-between"
              alignItems="flex-end"
            >
              <Stack>
                <Typography className="!font-semibold">{row.x.symbol}</Typography>
              </Stack>
              <Stack>
                <Typography className="text-gray-600 !text-sm !font-semibold">
                  Balance: {formatByIntl(xCoinInfo?.fixedBalance)}
                </Typography>
              </Stack>
            </Stack>
            <Stack justifyContent="center" spacing={2} direction="column" sx={{ pt: 1 }}>
              <FormControl>
                <TextField
                  label="Amount"
                  placeholder=""
                  value={xAmount}
                  inputMode="decimal"
                  autoComplete="off"
                  onChange={(e) => {
                    const value = e.target.value.replaceAll(',', '');
                    if (value === '') {
                      setXAmount('');
                      return;
                    }
                    if (/^\d*\.?\d*$/.test(value) === false) {
                      return;
                    }
                    const xBalance = xCoinInfo?.fixedBalance || 0;
                    if (Number(value) > xBalance) {
                      setXAmount(formatByIntl(xBalance));
                    } else {
                      setXAmount(formatByIntl(value));
                    }
                  }}
                  InputProps={{
                    endAdornment: (
                      <InputAdornment position="end">
                        <Stack direction="row" spacing={0.5}>
                          <Button
                            size="small"
                            variant="outlined"
                            onClick={() => {
                              setXAmount(
                                formatByIntl(
                                  new BigNumber(xCoinInfo?.fixedBalance || 0)
                                    .div(2)
                                    .toString()
                                )
                              );
                            }}
                          >
                            Half
                          </Button>
                          <Button
                            size="small"
                            variant="outlined"
                            onClick={() => {
                              setXAmount(
                                formatByIntl(
                                  (xCoinInfo?.fixedBalance || 0).toString()
                                )
                              );
                            }}
                          >
                            Max
                          </Button>
                        </Stack>
                      </InputAdornment>
                    ),
                  }}
                />
              </FormControl>
              <Stack
                direction="row"
                className="mb-2 w-full"
                justifyContent="space-between"
                alignItems="flex-end"
              >
                <Stack>
                  <Typography className="!font-semibold">{row.y.symbol}</Typography>
                </Stack>
                <Stack>
                  <Typography className="text-gray-600 !text-sm !font-semibold">
                    Balance: {formatByIntl(yCoinInfo?.fixedBalance)}
                  </Typography>
                </Stack>
              </Stack>
              <FormControl>
                <TextField
                  label={yLabelError || 'Automatic calculation'}
                  placeholder=""
                  value={yAmount}
                  inputMode="decimal"
                  autoComplete="off"
                  InputLabelProps={{ style: { color: yLabelError ? 'red' : 'inherit' } }}
                  onChange={(e) => {
                    setYAmount(e.target.value);
                  }}
                  InputProps={{
                    endAdornment: yLabelError && (
                      <InputAdornment position="end">
                        <Stack direction="row" spacing={0.5}>
                          <Button
                            size="small"
                            variant="outlined"
                            onClick={() => {
                              router.push('./swap-v2');
                            }}
                          >
                            Go to Swap
                          </Button>
                        </Stack>
                      </InputAdornment>
                    ),
                  }}
                />
              </FormControl>
            </Stack>

            <Box sx={{ pt: 2, mt: 2 }}>
              <span className="text-gray-400 text-sm mt-4 mr-2">Slippage</span>
              {[0.005, 0.01, 0.03].map((item, index) => (
                <Button
                  key={item.toString()}
                  variant={slippage === item ? 'contained' : 'outlined'}
                  size="small"
                  sx={{ mr: 1 }}
                  onClick={() => {
                    if (slippage === item) {
                      setSlippage(0);
                    } else {
                      setSlippage(item);
                      setCustomSlippage('');
                    }
                  }}
                >
                  {item * 100}%
                </Button>
              ))}
              <FormControl>
                <TextField
                  sx={{
                    width: '90px',
                    height: '30px',
                    '& .MuiInputBase-root': {
                      height: '30px',
                      fontSize: '0.875rem',
                    },
                  }}
                  placeholder="0"
                  id="outlined-basic"
                  value={customSlippage}
                  variant="outlined"
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                    setSlippage(0);
                    setCustomSlippage(e.target.value);
                  }}
                />
              </FormControl>
              <span className="text-gray-400 text-sm ml-1">%</span>
            </Box>
          </>
        );
      case 1:
        return (
          <>
            <Stack spacing={2} sx={{ mt: 1 }}>
              <Stack direction="row" justifyContent="space-between">
                <Stack direction="row" alignItems="center">
                  <Icon url={xCoinInfo?.icon_url || ''} />
                  <Icon url={yCoinInfo?.icon_url || ''} />
                  <Box className="text-gray-400 text-sm font-medium">{`${row.x.symbol}-${row.y.symbol} LP : `}</Box>
                </Stack>
                <Box sx={{ fontWeight: 'bold', fontSize: '1.2em', ml: 1 }}>
                  + {receive.liquidity}
                </Box>
              </Stack>
              <Stack
                direction="row"
                alignItems="center"
                justifyContent="space-between"
                spacing={0.5}
              >
                <Box className="text-gray-400 text-sm font-medium">Your share in the pair :</Box>
                <Box> {receive.share}%</Box>
              </Stack>
            </Stack>
            <Divider sx={{ borderStyle: 'dashed', borderColor: 'gray', my: 2 }} />
            <Typography sx={{ mt: 1 }}> Info</Typography>
            <Stack sx={{ mt: 2 }}>
              <Stack direction="column" gap={2}>
                <Stack direction="row" justifyContent="space-between">
                  <Stack direction="row" alignItems="center">
                    <Icon url={xCoinInfo?.icon_url || ''} />
                    {row.x.symbol}:
                  </Stack>
                  <span>- {xAmount}</span>
                </Stack>
                <Stack direction="row" justifyContent="space-between">
                  <Stack direction="row" alignItems="center">
                    <Icon url={yCoinInfo?.icon_url || ''} />
                    {row.y.symbol}:
                  </Stack>
                  <span>- {yAmount}</span>
                </Stack>
              </Stack>
              <Stack
                sx={{ mt: 1 }}
                direction="row"
                alignItems="center"
                justifyContent="space-between"
              >
                <Box className="text-gray-400 text-sm font-medium">Slippage :</Box>
                <Box> {slippage * 100}% </Box>
              </Stack>
            </Stack>
          </>
        );
      default:
        return <></>;
    }
  };

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle sx={{ pb: 2 }}>{STEPS[activeStep]}</DialogTitle>

      <DialogContent
        sx={{
          width: '480px',
          height: '280px',
          overflow: 'unset',
        }}
      >
        {getStepContent(activeStep)}
      </DialogContent>

      <DialogActions>
        <Button
          fullWidth
          variant="outlined"
          color="inherit"
          onClick={activeStep === 0 ? onClose : handleBack}
        >
          {activeStep === 0 ? 'Cancel' : 'Previous'}
        </Button>

        {activeStep === STEPS.length - 1 ? (
          <SessionKeyGuard onClick={handleAddLiquidity}>
            <LoadingButton fullWidth loading={isPending} variant="contained">
              Confirm
            </LoadingButton>
          </SessionKeyGuard>
        ) : (
          <LoadingButton
            fullWidth
            variant="contained"
            disabled={xAmount === '' || yAmount === '' || yLabelError !== undefined}
            onClick={handleNext}
          >
            Next
          </LoadingButton>
        )}
      </DialogActions>
    </Dialog>
  );
}
