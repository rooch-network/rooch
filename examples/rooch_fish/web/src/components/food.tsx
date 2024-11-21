import { Container, Graphics } from '@pixi/react';
import { useCallback } from 'react';
import * as PIXI from 'pixi.js';

interface FoodProps {
  x: number;
  y: number;
  size?: number;
  color?: number;
}

export const Food = ({ x, y, size = 6, color = 0x00FF00 }: FoodProps) => {
  const drawFood = useCallback((g: PIXI.Graphics) => {
    g.clear();
    g.beginFill(color);
    g.drawCircle(0, 0, size);
    g.endFill();
  }, [color, size]);

  return (
    <Container x={x} y={y}>
      <Graphics draw={drawFood} />
    </Container>
  );
};
