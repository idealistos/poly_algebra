import React, { useEffect, useRef } from 'react';
import { Layer } from 'react-konva';
import type { PlotPointElement, Shape, PlotData } from './types';
import { Html } from 'react-konva-utils';
import { PLOT_COLORS, transformPlotColor } from './utils';
import { ObjectType } from './enums';

interface CanvasPointLayerProps {
    plotDataByLocusName: Record<string, PlotData>;
    displayedPlotNames: Set<string>;
    shapes: Shape[]; // Use proper Shape type
}

const CanvasPointLayer: React.FC<CanvasPointLayerProps> = ({
    plotDataByLocusName,
    displayedPlotNames,
    shapes
}) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);


    useEffect(() => {
        // Helper function to get locus ordinal number
        const getLocusOrdinal = (locusName: string) => {
            const locusShapes = shapes.filter(shape =>
                shape.dbObject.object_type === ObjectType.Locus
            );
            return locusShapes.findIndex(shape => shape.dbObject.name === locusName) % 10;
        };

        const canvas = canvasRef.current;
        if (!canvas) return;

        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        // Clear canvas
        ctx.clearRect(0, 0, canvas.width, canvas.height);

        // Render all visible points
        Array.from(displayedPlotNames).forEach(locusName => {
            const plotData = plotDataByLocusName[locusName];
            if (!plotData) return;

            const points = plotData.points;
            if (!points) return;

            // Get the target color for this locus
            const locusOrdinal = getLocusOrdinal(locusName);
            const targetColor = PLOT_COLORS[locusOrdinal];

            points.forEach((point: PlotPointElement[]) => {
                // Coordinates are already physical and in visible range
                const x = point[0] as number;
                const y = point[1] as number;
                const redColor = point[2] as { r: number; g: number; b: number };

                // Transform the red interpolated color to use the target hue
                const transformedColor = transformPlotColor(redColor, targetColor);

                ctx.fillStyle = `rgb(${transformedColor.r}, ${transformedColor.g}, ${transformedColor.b})`;
                ctx.globalAlpha = 0.8;
                ctx.fillRect(x, y, 1, 1);
            });
        });
    }, [plotDataByLocusName, displayedPlotNames, shapes]);

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