import React, { useEffect, useRef } from 'react';
import { Layer } from 'react-konva';
import type { PlotPointElement } from './types';
import { Html } from 'react-konva-utils';

interface CanvasPointLayerProps {
    plotPointsByLocusName: Record<string, PlotPointElement[][]>;
    displayedPlotNames: Set<string>;
}

const CanvasPointLayer: React.FC<CanvasPointLayerProps> = ({
    plotPointsByLocusName,
    displayedPlotNames
}) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        // Clear canvas
        ctx.clearRect(0, 0, canvas.width, canvas.height);

        // Render all visible points
        Array.from(displayedPlotNames).forEach(locusName => {
            const points = plotPointsByLocusName[locusName];
            if (!points) return;

            points.forEach(point => {
                // Coordinates are already physical and in visible range
                const x = point[0] as number;
                const y = point[1] as number;
                const color = point[2] as { r: number; g: number; b: number };

                ctx.fillStyle = `rgb(${color.r}, ${color.g}, ${color.b})`;
                ctx.globalAlpha = 0.8;
                ctx.fillRect(x, y, 1, 1);
            });
        });
    }, [plotPointsByLocusName, displayedPlotNames]);

    return (
        <Layer>
            <Html>
                <canvas
                    ref={canvasRef}
                    width={window.innerWidth}
                    height={window.innerHeight}
                    style={{
                        position: 'absolute',
                        top: 0,
                        left: 0,
                        pointerEvents: 'none',
                        zIndex: 1000
                    }}
                />
            </Html>
        </Layer>
    );
};

export default CanvasPointLayer; 