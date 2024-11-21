// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { Container, Graphics, Stage } from '@pixi/react';
import { useMemo, useState } from 'react';
import { BlurFilter, ColorMatrixFilter } from 'pixi.js';
import { Box, Button, Paper, Typography, Grid,  AppBar, Toolbar } from '@mui/material';
import { useSnackbar } from 'notistack';
import { Args, Transaction } from "@roochnetwork/rooch-sdk";
import {
  UseSignAndExecuteTransaction,
} from "@roochnetwork/rooch-sdk-kit";
import { Fish } from '../components/fish';
import { Food } from '../components/food';
import { useFishController } from '../hooks/useFishController';
import { usePondState } from '../hooks/usePondState';
import { usePlayerState } from '../hooks/usePlayerState';
import { config } from "../config/index";
import { ExitZone } from '../types/index';

const RedColor = 0xFF6B6B
const BlueColor = 0x0000FF
const YellowColor = 0xFFFF00
const GreenColor = 0x00FF00

export const PondScene = () => {
  const { enqueueSnackbar } = useSnackbar();
  const [purchaseLoading, setPurchaseLoading] = useState(false);
  const [feedLoading, setFeedLoading] = useState(false);
  const [exitLoading, setExitLoading] = useState(false);
  const { data: pondState, fishData, foodData, getRecentDelays  } = usePondState(0);
  const { fish_ids } = usePlayerState(0)
  
  const width = 800;
  const height = 800;

  const scale = useMemo(() => {
    if (!pondState) return 1;
    
    const horizontalScale = (width - 80) / pondState.width;
    const verticalScale = (height - 80) / pondState.height;
    
    // Use the smaller scale to ensure pond fits within boundaries
    return Math.min(horizontalScale, verticalScale); 
  }, [pondState, width, height]);

  const boundaries = useMemo(() => ({
    minX: 40,
    maxX: width - 80,
    minY: 40,
    maxY: height - 80
  }), [width, height]);

  const waterFilters = useMemo(() => {
    const blur = new BlurFilter(2);
    const colorMatrix = new ColorMatrixFilter();
    
    colorMatrix.brightness(1.1, true);
    return [blur, colorMatrix];
  }, []);

  const foodItems = useMemo(() => {
    const items = [];
    const foodColors = [0x00FF00, 0xFFFF00, 0xFFA500];
    const margin = 40;
    
    for (let i = 0; i < 10; i++) {
      items.push({
        x: margin + Math.random() * (width - margin * 2),
        y: margin + Math.random() * (height - margin * 2),
        size: 4 + Math.random() * 4,
        color: foodColors[Math.floor(Math.random() * foodColors.length)]
      });
    }
    return items;
  }, [width, height]);

  const playerFirstFish = useMemo(() => {
    if (!fish_ids || !fishData || fish_ids.length === 0) return null;
    return fishData.find((fish: any) => fish.id === fish_ids[0]);
  }, [fish_ids, fishData]);

  useFishController(0, playerFirstFish ? parseInt((playerFirstFish as any).id) : 0, 100, 100, boundaries);

  const { mutateAsync: signAndExecuteTransaction } =
      UseSignAndExecuteTransaction();

  const handlePurchaseFish = async () => {
    try {
      setPurchaseLoading(true);
      const txn = new Transaction();
      txn.callFunction({
        address: config.roochFishAddress,
        module: "rooch_fish", 
        function: "purchase_fish",
        args: [
          Args.objectId(config.gameStateObjectID),
          Args.u64(BigInt(0)), // pond_id 
        ],
      });
  
      await signAndExecuteTransaction({ transaction: txn });
      setPurchaseLoading(false);
    } catch (error) {
      console.error(String(error));

      if (String(error).includes("1004")) {
        enqueueSnackbar("Insufficient gas, please claim gas first", { 
          variant: "warning" 
        });
      } else {
        enqueueSnackbar(String(error), {
          variant: "warning"
        });
      }
    } finally {
      setPurchaseLoading(false);
    }
  };

  const handleFeedPond = async () => {
    try {
      setFeedLoading(true);
      const txn = new Transaction();
      txn.callFunction({
        address: config.roochFishAddress,
        module: "rooch_fish",
        function: "feed_food",
        args: [
          Args.objectId(config.gameStateObjectID),
          Args.u64(BigInt(0)), // pond_id
          Args.u64(BigInt(10)), // count
        ],
      });

      const tx = await signAndExecuteTransaction({ transaction: txn });
      if (tx?.output?.status?.type != 'executed') {
        const errorMsg = `Feed food failed: ${tx?.output?.status?.type}`;
        console.error("Feed food fail:", errorMsg);
        enqueueSnackbar(errorMsg, { 
          variant: "warning" 
        });
        return;
      }

      console.log("Feed food success!")
      enqueueSnackbar("Successfully fed the pond!", { variant: "success" });
    } catch (error) {
      console.error(String(error));
      enqueueSnackbar(String(error), { variant: "error" });
    } finally {
      setFeedLoading(false);
    }
  };

  const isInExitZone = (fishX: number, fishY: number): boolean => {
    if (!pondState?.exit_zones) return false;
    
    return pondState.exit_zones.some((zone: ExitZone) => {
      const distance = Math.sqrt(
        Math.pow(fishX - zone.x, 2) + 
        Math.pow(fishY - zone.y, 2)
      );
      return distance <= zone.radius;
    });
  };
  
  const handleExitPond = async () => {
    if (!playerFirstFish) return;
    
    if (!isInExitZone(playerFirstFish.x, playerFirstFish.y)) {
      enqueueSnackbar("Fish must be in an exit zone to leave the pond", { 
        variant: "warning" 
      });
      return;
    }
  
    try {
      setExitLoading(true);
      const txn = new Transaction();
      txn.callFunction({
        address: config.roochFishAddress,
        module: "rooch_fish",
        function: "destroy_fish",
        args: [
          Args.objectId(config.gameStateObjectID),
          Args.u64(BigInt(0)), // pond_id
          Args.u64(BigInt(playerFirstFish.id)),
        ],
      });
  
      const tx = await signAndExecuteTransaction({ transaction: txn });
      if (tx?.output?.status?.type != 'executed') {
        throw new Error(`Exit failed: ${tx?.output?.status?.type}`);
      }
  
      enqueueSnackbar("Successfully exited the pond!", { 
        variant: "success" 
      });
    } catch (error) {
      console.error(String(error));
      enqueueSnackbar(String(error), { 
        variant: "error" 
      });
    } finally {
      setExitLoading(false);
    }
  };

  // Add this utility function at the bottom of the file or in utils/time.ts
  const getAverageDelay = (delays: any[]) => {
    if (!delays || delays.length === 0) return null;
    const sum = delays.reduce((acc, curr) => {
      return {
        totalDelay: acc.totalDelay + curr.totalDelay,
        syncDelay: acc.syncDelay + curr.syncDelay
      };
    }, { totalDelay: 0, syncDelay: 0 });
    
    return {
      confirmDelay: Math.round((sum.totalDelay - sum.syncDelay) / delays.length),
      syncDelay: Math.round(sum.syncDelay / delays.length)
    };
  };

  return (
    <Box>
      <AppBar position="static" color="transparent" elevation={0} sx={{ mb: 2 }}>
        <Toolbar>
          <Button
            variant="contained"
            color="primary"
            onClick={handleFeedPond}
            disabled={feedLoading}
            sx={{ mr: 2 }}
          >
            {feedLoading ? 'Feeding...' : 'Feed 10 food'}
          </Button>

          {playerFirstFish && (
            <Button
              variant="contained"
              color="secondary"
              onClick={handleExitPond}
              disabled={exitLoading}
            >
              {exitLoading ? 'Exiting...' : 'Exit Pond'}
            </Button>
          )}
        </Toolbar>
      </AppBar>

      <Box position="relative">
        <Stage 
          width={width} 
          height={height} 
          options={{ 
            backgroundColor: 0xADD8E6,
            antialias: true 
          }}
        >
          <Container>
            <Graphics
              draw={g => {
                g.clear();
                g.beginFill(0x4FA4FF, 0.3);
                g.drawRoundedRect(20, 20, width - 40, height - 40, 15);
                g.endFill();
              }}
              filters={waterFilters}
            />
            
            <Graphics
              draw={g => {
                g.clear();
                g.lineStyle(4, 0x2980B9);
                g.drawRoundedRect(20, 20, width - 40, height - 40, 15);
              }}
            />

            <Container name="fishContainer" x={20} y={20} width={2000} height={2000}>
              <>
                {pondState?.exit_zones?.map((zone: ExitZone, index: number) => (
                    <Graphics
                      key={`exit-zone-${index}`}
                      draw={g => {
                        g.clear();
                        g.beginFill(GreenColor, 0.3);
                        g.drawCircle(
                          40 + zone.x * scale,
                          40 + zone.y * scale,
                          zone.radius * scale / 2
                        );
                        g.endFill();
                      }}
                    />
                  ))}

                  {fishData && fishData.map((fishState: any, index: number) => (
                      <Fish 
                        key={`fish-${index}`}
                        x={40 + fishState.x * scale} 
                        y={40 + fishState.y * scale} 
                        rotation={0}
                        scale={fishState.size / 7} 
                        color={playerFirstFish?.id == fishState.id ? BlueColor : RedColor}
                      />
                  ))}

                  {foodData && foodData.map((food: any, index: number) => (
                    <Food
                      key={`food-${index}`}
                      x={40 + food.x * scale}
                      y={40 + food.y * scale}
                      size={food.size * scale / 2}
                      color={YellowColor}
                    />
                  ))}
                </>
            </Container>
          </Container>
        </Stage>

        {(!fish_ids || fish_ids.length==0) && (
          <Paper
            sx={{
              position: 'absolute',
              top: '50%',
              left: '50%',
              transform: 'translate(-50%, -50%)',
              padding: 3,
              textAlign: 'center',
              backgroundColor: 'rgba(255, 255, 255, 0.9)',
              backdropFilter: 'blur(4px)',
            }}
          >
            <Typography variant="h6" gutterBottom>
              Welcome to the Pond!
            </Typography>
            <Typography variant="body1" sx={{ mb: 2 }}>
              You don't have a fish yet. Purchase one to start playing!
            </Typography>
            <Button
              variant="contained"
              color="primary"
              onClick={handlePurchaseFish}
              disabled={purchaseLoading}
            >
              {purchaseLoading ? 'Purchasing...' : 'Purchase Fish'}
            </Button>
          </Paper>
        )}
      </Box>

      {playerFirstFish && (
        <Paper 
          sx={{ 
            mt: 2,
            p: 2,
            backgroundColor: 'rgba(255, 255, 255, 0.9)'
          }}
        >
          <Grid container spacing={2} alignItems="center">
            <Grid item xs={4}>
              <Typography>
                Position: ({Math.round(playerFirstFish.x)}, {Math.round(playerFirstFish.y)})
              </Typography>
            </Grid>
            <Grid item xs={4}>
              <Typography>
                Confirm: {(() => {
                  const delays = getRecentDelays();
                  const avg = getAverageDelay(delays);
                  return avg ? `${avg.confirmDelay}ms` : 'N/A';
                })()}
              </Typography>
            </Grid>
            <Grid item xs={4}>
              <Typography>
                Sync: {(() => {
                  const delays = getRecentDelays();
                  const avg = getAverageDelay(delays);
                  return avg ? `${avg.syncDelay}ms` : 'N/A';
                })()}
              </Typography>
            </Grid>
          </Grid>
        </Paper>
      )}
    </Box>
  );
};
