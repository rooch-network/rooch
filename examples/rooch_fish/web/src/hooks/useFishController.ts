import { config } from "../config/index";
import { useState, useEffect, useRef } from 'react';
import { Args, Transaction } from "@roochnetwork/rooch-sdk";
import { useSnackbar } from 'notistack';
import { useSignAndExecuteTransaction } from "./useSignAndExecuteTransaction";

interface FishState {
  x: number;
  y: number;
  rotation: number;
  velocity: {
    x: number;
    y: number;
  };
}

interface BoundaryProps {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
}

export const useFishController = (pondID:number, fishID: number, initialX: number, initialY: number, boundaries: BoundaryProps) => {
  const [fishState, setFishState] = useState<FishState>({
    x: initialX,
    y: initialY,
    rotation: 0,
    velocity: { x: 0, y: 0 }
  });

  const isMoving = useRef(false);
  const { enqueueSnackbar } = useSnackbar();

  const { mutateAsync: signAndExecuteTransaction } =
    useSignAndExecuteTransaction();

  const handleFishMove = async (direction: number) => {
    if (isMoving.current) {
      return;
    }

    isMoving.current = true;
    console.log("move fish start, with direction:", direction, "fish_id:", fishID)
    
    setFishState(prev => ({ ...prev, error: undefined }));

    try {
      const txn = new Transaction();
      txn.callFunction({
        address: config.roochFishAddress,
        module: "rooch_fish", 
        function: "move_fish",
        args: [
          Args.objectId(config.gameStateObjectID),
          Args.u64(BigInt(pondID)),
          Args.u64(BigInt(fishID)),
          Args.u8(direction),
        ],
      });
  
      (txn as any).data.maxGas = BigInt(50000000 * 10)

      const tx = await signAndExecuteTransaction({ transaction: txn });
      if (tx?.output?.status?.type != 'executed') {
        const errorMsg = `Move failed: ${tx?.output?.status?.type}`;
        console.error("move fish fail:", tx?.output?.status);
        enqueueSnackbar(errorMsg, { 
          variant: "warning" 
        });
        return;
      }

      console.log("move fish success, tx:", tx, "tx_order:", tx.sequence_info.tx_order);
    } catch (error) {
      console.error("move fish error:", error);

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
      isMoving.current = false;
    }
  };

  const speed = 5;

  useEffect(() => {
    const keys = new Set<string>();

    const handleKeyDown = (e: KeyboardEvent) => {
      keys.add(e.key);
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      keys.delete(e.key);
    };

    const updatePosition = () => {
      setFishState(prev => {
        let dx = 0;
        let dy = 0;

        if (keys.has('ArrowLeft') || keys.has('a')) {
          dx -= speed;
          handleFishMove(3);
        }

        if (keys.has('ArrowRight') || keys.has('d')) {
          dx += speed;
          handleFishMove(1);
        }

        if (keys.has('ArrowUp') || keys.has('w')) {
          dy -= speed;
          handleFishMove(2);
        }

        if (keys.has('ArrowDown') || keys.has('s')) {
          dy += speed;
          handleFishMove(0);
        }

        let newX = Math.max(boundaries.minX, Math.min(boundaries.maxX, prev.x + dx));
        let newY = Math.max(boundaries.minY, Math.min(boundaries.maxY, prev.y + dy));
        
        let newRotation = prev.rotation;
        if (dx !== 0 || dy !== 0) {
          newRotation = Math.atan2(dy, dx);
        }

        return {
          x: newX,
          y: newY,
          rotation: newRotation,
          velocity: { x: dx, y: dy }
        };
      });
    };

    const gameLoop = setInterval(updatePosition, 30);

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);

    return () => {
      clearInterval(gameLoop);
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [pondID, fishID, boundaries, speed]);

  return fishState;
};
