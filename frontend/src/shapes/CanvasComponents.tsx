import React from 'react';
import { Circle, Line, Text, Path } from "react-konva";
import type { Shape } from '../types';
import { getColor, ShapeState } from '../enums';
import type { Vector2d } from 'konva/lib/types';

export function CanvasFixedPoint({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    const coords = shape.points[0];
    if (!coords) return null;
    const { px, py } = getPhysicalCoords(coords);
    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;

    return (
        <>
            {/* Glow */}
            {isSuggested && (
                <Circle
                    x={px}
                    y={py}
                    radius={14}
                    fill="#ffb6c1"
                    opacity={0.4}
                />
            )}
            <Circle x={px} y={py} radius={8} stroke={color} strokeWidth={2} />
            <Circle x={px} y={py} radius={3} fill={color} />
            <Line points={[px, py - 8, px, py - 12]} stroke={color} strokeWidth={2} />
            <Line points={[px, py + 8, px, py + 12]} stroke={color} strokeWidth={2} />
            <Line points={[px - 8, py, px - 12, py]} stroke={color} strokeWidth={2} />
            <Line points={[px + 8, py, px + 12, py]} stroke={color} strokeWidth={2} />
            <Text x={px + 10} y={py - 25} text={shape.dbObject.name} fontSize={16} fill={color} />
        </>
    );
}

export function CanvasFreePoint({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    const coords = shape.points[0];
    if (!coords) return null;
    const { px, py } = getPhysicalCoords(coords);
    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;
    return (
        <>
            {/* Glow */}
            {isSuggested && (
                <Circle
                    x={px}
                    y={py}
                    radius={14}
                    fill="#ffb6c1"
                    opacity={0.4}
                />
            )}
            <Circle x={px} y={py} radius={8} stroke={color} strokeWidth={2} />
            <Circle x={px} y={py} radius={4} fill={color} />
            <Text x={px + 10} y={py - 25} text={shape.dbObject.name} fontSize={16} fill={color} />
        </>
    );
}

export function CanvasInitialPoint({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    const coords = shape.points[0];
    if (!coords) return null;
    const { px, py } = getPhysicalCoords(coords);
    const color = getColor(shape);

    return (
        <>
            <Circle x={px} y={py} radius={5} fill={color} />
            <Text x={px + 10} y={py - 25} text={shape.dbObject.name} fontSize={16} fill={color} />
        </>
    );
}

export function CanvasMidpoint({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    if (shape.points.length < 2) return null;

    const point1 = getPhysicalCoords(shape.points[0]);
    const point2 = getPhysicalCoords(shape.points[1]);

    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;

    // Calculate midpoint
    const midX = (point1.px + point2.px) / 2;
    const midY = (point1.py + point2.py) / 2;

    // Calculate 1/4 and 3/4 points
    const quarterX = point1.px + (point2.px - point1.px) * 0.25;
    const quarterY = point1.py + (point2.py - point1.py) * 0.25;
    const threeQuarterX = point1.px + (point2.px - point1.px) * 0.75;
    const threeQuarterY = point1.py + (point2.py - point1.py) * 0.75;

    // Calculate directions
    const dx = point2.px - point1.px;
    const dy = point2.py - point1.py;
    const length = Math.sqrt(dx * dx + dy * dy);

    if (length === 0) return null; // Avoid division by zero

    // Normalize segment direction
    const segX = dx / length;
    const segY = dy / length;

    // Normalize and rotate 90 degrees to get perpendicular direction
    const perpX = -dy / length;
    const perpY = dx / length;

    // Calculate mark positions (5 pixels long along segment direction, 3 pixels apart perpendicular)
    const markHalfLength = 2; // 1.5 pixels in each direction along segment
    const markOffset = 3.5; // 2.5 pixels perpendicular to segment

    // Marks at 1/4 point - on parallel lines 3 pixels apart
    const quarterMark1X1 = quarterX + perpX * markOffset - segX * markHalfLength;
    const quarterMark1Y1 = quarterY + perpY * markOffset - segY * markHalfLength;
    const quarterMark1X2 = quarterX - perpX * markOffset - segX * markHalfLength;
    const quarterMark1Y2 = quarterY - perpY * markOffset - segY * markHalfLength;

    const quarterMark2X1 = quarterX + perpX * markOffset + segX * markHalfLength;
    const quarterMark2Y1 = quarterY + perpY * markOffset + segY * markHalfLength;
    const quarterMark2X2 = quarterX - perpX * markOffset + segX * markHalfLength;
    const quarterMark2Y2 = quarterY - perpY * markOffset + segY * markHalfLength;

    // Marks at 3/4 point - on parallel lines 3 pixels apart
    const threeQuarterMark1X1 = threeQuarterX + perpX * markOffset - segX * markHalfLength;
    const threeQuarterMark1Y1 = threeQuarterY + perpY * markOffset - segY * markHalfLength;
    const threeQuarterMark1X2 = threeQuarterX - perpX * markOffset - segX * markHalfLength;
    const threeQuarterMark1Y2 = threeQuarterY - perpY * markOffset - segY * markHalfLength;

    const threeQuarterMark2X1 = threeQuarterX + perpX * markOffset + segX * markHalfLength;
    const threeQuarterMark2Y1 = threeQuarterY + perpY * markOffset + segY * markHalfLength;
    const threeQuarterMark2X2 = threeQuarterX - perpX * markOffset + segX * markHalfLength;
    const threeQuarterMark2Y2 = threeQuarterY - perpY * markOffset + segY * markHalfLength;

    return (
        <>
            {/* Glow - pink circle centered on midpoint */}
            {isSuggested && (
                <Circle
                    x={midX}
                    y={midY}
                    radius={7}
                    fill="#ffb6c1"
                    opacity={0.4}
                />
            )}
            {/* Dotted segment connecting endpoints */}
            <Line
                points={[point1.px, point1.py, point2.px, point2.py]}
                stroke={color}
                strokeWidth={1}
                dash={[5, 5]}
            />

            {/* Endpoint circles */}
            <Circle x={point1.px} y={point1.py} radius={3} fill={color} />
            <Circle x={point2.px} y={point2.py} radius={3} fill={color} />

            {/* Midpoint circle */}
            <Circle x={midX} y={midY} radius={2} fill={color} />

            {/* Marks at 1/4 point */}
            <Line
                points={[quarterMark1X1, quarterMark1Y1, quarterMark1X2, quarterMark1Y2]}
                stroke={color}
                strokeWidth={1}
            />
            <Line
                points={[quarterMark2X1, quarterMark2Y1, quarterMark2X2, quarterMark2Y2]}
                stroke={color}
                strokeWidth={1}
            />

            {/* Marks at 3/4 point */}
            <Line
                points={[threeQuarterMark1X1, threeQuarterMark1Y1, threeQuarterMark1X2, threeQuarterMark1Y2]}
                stroke={color}
                strokeWidth={1}
            />
            <Line
                points={[threeQuarterMark2X1, threeQuarterMark2Y1, threeQuarterMark2X2, threeQuarterMark2Y2]}
                stroke={color}
                strokeWidth={1}
            />

            {/* Label */}
            <Text x={midX + 10} y={midY - 25} text={shape.dbObject.name} fontSize={16} fill={color} />
        </>
    );
}

export function CanvasLineAB({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    if (shape.points.length < 2) return null;

    const point1 = getPhysicalCoords(shape.points[0]);
    const point2 = getPhysicalCoords(shape.points[1]);

    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;

    // Calculate the full line that extends across the visible area
    const width = window.innerWidth;
    const height = window.innerHeight;

    // Calculate line equation: y = mx + b
    const dx = point2.px - point1.px;
    const dy = point2.py - point1.py;

    if (dx === 0) {
        // Vertical line
        const x = point1.px;
        const linePoints = [x, 0, x, height];

        return (
            <>
                {/* Glow - pink background line */}
                {isSuggested && (
                    <Line
                        points={linePoints}
                        stroke="#ffb6c1"
                        strokeWidth={8}
                        opacity={0.4}
                    />
                )}
                {/* Full dotted line */}
                <Line
                    points={linePoints}
                    stroke={color}
                    strokeWidth={1}
                    dash={[5, 5]}
                />
                {/* Solid segment between points */}
                <Line
                    points={[x, point1.py, x, point2.py]}
                    stroke={color}
                    strokeWidth={2}
                />
                <Circle x={point1.px} y={point1.py} radius={3} fill={color} />
                <Circle x={point2.px} y={point2.py} radius={3} fill={color} />
                <Text
                    x={x + 10}
                    y={(point1.py + point2.py) / 2 - 20}
                    text={shape.dbObject.name}
                    fontSize={16}
                    fill={color}
                />
            </>
        );
    }

    const m = dy / dx;
    const b = point1.py - m * point1.px;

    // Find intersection points with canvas boundaries
    let x1 = 0, y1 = b;
    let x2 = width, y2 = m * width + b;

    // Check if line intersects with top/bottom boundaries
    if (y1 < 0) {
        x1 = -b / m;
        y1 = 0;
    } else if (y1 > height) {
        x1 = (height - b) / m;
        y1 = height;
    }

    if (y2 < 0) {
        x2 = -b / m;
        y2 = 0;
    } else if (y2 > height) {
        x2 = (height - b) / m;
        y2 = height;
    }

    // Ensure points are within canvas bounds
    x1 = Math.max(0, Math.min(width, x1));
    x2 = Math.max(0, Math.min(width, x2));
    y1 = Math.max(0, Math.min(height, y1));
    y2 = Math.max(0, Math.min(height, y2));

    return (
        <>
            {/* Glow - pink background line */}
            {isSuggested && (
                <Line
                    points={[x1, y1, x2, y2]}
                    stroke="#ffb6c1"
                    strokeWidth={8}
                    opacity={0.4}
                />
            )}
            {/* Full dotted line */}
            <Line
                points={[x1, y1, x2, y2]}
                stroke={color}
                strokeWidth={1}
                dash={[5, 5]}
            />
            {/* Solid segment between points */}
            <Line
                points={[point1.px, point1.py, point2.px, point2.py]}
                stroke={color}
                strokeWidth={2}
            />
            <Circle x={point1.px} y={point1.py} radius={3} fill={color} />
            <Circle x={point2.px} y={point2.py} radius={3} fill={color} />
            <Text
                x={(point1.px + point2.px) / 2 + 10}
                y={(point1.py + point2.py) / 2 - 20}
                text={shape.dbObject.name}
                fontSize={16}
                fill={color}
            />
        </>
    );
}

export function CanvasLocus({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    const coords = shape.points[0];
    if (!coords) return null;
    const { px, py } = getPhysicalCoords(coords);
    const color = shape.state == ShapeState.Hinted ? "lightgray" : "red";
    return <Circle x={px} y={py} radius={6} stroke={color} strokeWidth={4} />;
}


export function CanvasIntersectionPoint({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    const coords = shape.points[0];
    if (!coords) return null;
    const { px, py } = getPhysicalCoords(coords);
    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;
    const isHinted = shape.state === ShapeState.Hinted;

    return (
        <>
            {/* Glow - similar to FreePoint */}
            {isSuggested && (
                <Circle
                    x={px}
                    y={py}
                    radius={14}
                    fill="#ffb6c1"
                    opacity={0.4}
                />
            )}
            {/* Main circle - identical to CanvasInitialPoint */}
            <Circle x={px} y={py} radius={5} fill={color} />
            {/* X mark for hinted state */}
            {isHinted && (
                <>
                    <Line
                        points={[px - 8, py - 8, px + 8, py + 8]}
                        stroke={color}
                        strokeWidth={2}
                    />
                    <Line
                        points={[px - 8, py + 8, px + 8, py - 8]}
                        stroke={color}
                        strokeWidth={2}
                    />
                </>
            )}
            <Text x={px + 10} y={py - 25} text={shape.dbObject.name} fontSize={16} fill={color} />
        </>
    );
}

export function CanvasSlidingPoint({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    const coords = shape.points[0];
    if (!coords) return null;
    const { px, py } = getPhysicalCoords(coords);
    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;
    const isHinted = shape.state === ShapeState.Hinted;

    // Get the line direction from the shape
    const slidingPointShape = shape as { lineDirection?: Vector2d };
    const lineDirection = slidingPointShape.lineDirection || { x: 1, y: 0 };

    // Calculate the angle of the line direction
    const angle = Math.atan2(-lineDirection.y, lineDirection.x);

    // Parameters
    const r = 7;
    const alpha = 0.5;

    // Calculate arc start and end points
    const arcStartX1 = px + r * Math.cos(angle + alpha);
    const arcStartY1 = py + r * Math.sin(angle + alpha);
    const arcEndX1 = px + r * Math.cos(Math.PI + angle - alpha);
    const arcEndY1 = py + r * Math.sin(Math.PI + angle - alpha);

    const arcStartX2 = px + r * Math.cos(angle + alpha + Math.PI);
    const arcStartY2 = py + r * Math.sin(angle + alpha + Math.PI);
    const arcEndX2 = px + r * Math.cos(2 * Math.PI + angle - alpha);
    const arcEndY2 = py + r * Math.sin(2 * Math.PI + angle - alpha);

    // Create SVG path for upper segment
    // Move to arc start, draw arc to end, then line back to start
    const pathData1 = `M ${arcStartX1} ${arcStartY1} A ${r} ${r} 0 0 1 ${arcEndX1} ${arcEndY1} L ${arcStartX1} ${arcStartY1} Z`;
    const pathData2 = `M ${arcStartX2} ${arcStartY2} A ${r} ${r} 0 0 1 ${arcEndX2} ${arcEndY2} L ${arcStartX2} ${arcStartY2} Z`;

    // Calculate points for the dotted line through the center
    const dottedLineLength = r + 8; // Extend beyond the circle
    const dottedLineStartX = px - Math.cos(angle) * dottedLineLength;
    const dottedLineStartY = py - Math.sin(angle) * dottedLineLength;
    const dottedLineEndX = px + Math.cos(angle) * dottedLineLength;
    const dottedLineEndY = py + Math.sin(angle) * dottedLineLength;

    return (
        <>
            {/* Glow */}
            {isSuggested && (
                <Circle
                    x={px}
                    y={py}
                    radius={14}
                    fill="#ffb6c1"
                    opacity={0.4}
                />
            )}

            {/* Upper segment */}
            <Path
                data={pathData1}
                fill={color}
                stroke={color}
                strokeWidth={1}
            />

            {/* Lower segment */}
            <Path
                data={pathData2}
                fill={color}
                stroke={color}
                strokeWidth={1}
            />

            {/* Gray dotted line through center when hinted */}
            {isHinted && (
                <Line
                    points={[dottedLineStartX, dottedLineStartY, dottedLineEndX, dottedLineEndY]}
                    stroke="gray"
                    strokeWidth={1}
                    dash={[2, 2]}
                />
            )}

            {/* Label */}
            <Text x={px + 10} y={py - 25} text={shape.dbObject.name} fontSize={16} fill={color} />
        </>
    );
}