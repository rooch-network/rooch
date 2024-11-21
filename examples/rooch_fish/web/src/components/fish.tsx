import { Container, Graphics } from '@pixi/react';
import { useCallback } from 'react';
import * as PIXI from 'pixi.js';

interface FishProps {
  x: number;
  y: number;
  scale?: number;
  rotation?: number;
  color?: number;
}

export const Fish = ({ x, y, scale = 1, rotation = 0, color = 0xFF6B6B}: FishProps) => {
  const drawFish = useCallback((g: PIXI.Graphics) => {
    g.clear();
    g.lineStyle(2, 0x000000, 1);
    g.beginFill(color);
    
    // Almost circular body
    g.moveTo(-15, 0);
    g.bezierCurveTo(-15, -15, 15, -15, 15, 0);
    g.bezierCurveTo(15, 15, -15, 15, -15, 0);
    
    // Small tail
    g.moveTo(-15, 0);
    g.lineTo(-22, -6);
    g.lineTo(-22, 6);
    g.lineTo(-15, 0);
    
    g.endFill();
  }, [color]);

  return (
    <Container x={x} y={y} scale={scale} rotation={rotation}>
      <Graphics draw={drawFish} />
    </Container>
  );
};
