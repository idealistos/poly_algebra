import React from 'react';
import { Circle, Line, Text, Path } from "react-konva";
import type { Shape } from '../types';
import { getColor, ShapeState } from '../enums';
import type { Vector2d } from 'konva/lib/types';
import { PLOT_COLORS } from '../utils';
import type { LocusShape } from './LocusShape';
import type { TwoLineAngleInvariantShape } from './TwoLineAngleInvariantShape';
import type { PpBisectorShape } from './PpBisectorShape';
import type { PpToLineShape } from './PpToLineShape';
import type { PlToLineShape } from './PlToLineShape';
import type { SlidingPointShape } from './SlidingPointShape';

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

export function CanvasPpBisector({ shape, getPhysicalCoords }: { shape: PpBisectorShape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    if (shape.points.length < 2) return null;

    const point1 = getPhysicalCoords(shape.points[0]);
    const point2 = getPhysicalCoords(shape.points[1]);

    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;
    const isDefaultOrSuggested = shape.state === ShapeState.Default || shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;

    // Get the defined line (perpendicular bisector)
    const definedLine = shape.getDefinedLine();
    if (!definedLine) return null;

    // Convert the defined line to physical coordinates
    const bisectorPoint = getPhysicalCoords(definedLine.point);
    const bisectorNormal = definedLine.n;

    // Calculate perpendicular symbol directions
    const segDx = point2.px - point1.px;
    const segDy = point2.py - point1.py;
    const segLength = Math.sqrt(segDx * segDx + segDy * segDy);

    let segDirX = 0, segDirY = 0, perpDirX = 0, perpDirY = 0;
    if (segLength > 0) {
        // Normalize the segment direction
        segDirX = segDx / segLength;
        segDirY = segDy / segLength;

        // Perpendicular direction (perpendicular to segment, parallel to bisector)
        perpDirX = -segDirY;
        perpDirY = segDirX;
    }

    const symbolLength = 10;

    // Calculate perpendicular symbol lines at the bisector point
    let symbolLines = null;
    if (segLength > 0) {
        // Calculate points for the perpendicular symbol at the bisector point
        const bp = bisectorPoint;
        const segOffset = segDirX * symbolLength;
        const perpOffset = perpDirX * symbolLength;

        // First line: from bp + segDir * 7 to bp + segDir * 7 + perpDir * 7
        const line1StartX = bp.px + segOffset;
        const line1StartY = bp.py + segDirY * symbolLength;
        const line1EndX = bp.px + segOffset + perpOffset;
        const line1EndY = bp.py + segDirY * symbolLength + perpDirY * symbolLength;

        // Second line: from bp + perpDir * 7 to bp + segDir * 7 + perpDir * 7
        const line2StartX = bp.px + perpOffset;
        const line2StartY = bp.py + perpDirY * symbolLength;
        const line2EndX = bp.px + segOffset + perpOffset;
        const line2EndY = bp.py + segDirY * symbolLength + perpDirY * symbolLength;

        symbolLines = {
            line1: [line1StartX, line1StartY, line1EndX, line1EndY],
            line2: [line2StartX, line2StartY, line2EndX, line2EndY]
        };
    }

    // Calculate the full bisector line that extends across the visible area
    const width = window.innerWidth;
    const height = window.innerHeight;

    // Calculate line equation for the bisector: y = mx + b
    // Convert normal vector to direction vector: rotate by 90 degrees
    // Note: y-axis is flipped in physical coordinates, so we use bisectorNormal.y directly
    const dx = bisectorNormal.y;
    const dy = bisectorNormal.x;

    if (dx === 0) {
        // Vertical bisector
        const x = bisectorPoint.px;
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
                {/* Solid bisector line */}
                <Line
                    points={linePoints}
                    stroke={color}
                    strokeWidth={2}
                />
                {/* Horizontal dotted line joining the two points */}
                <Line
                    points={[point1.px, point1.py, point2.px, point2.py]}
                    stroke={isDefaultOrSuggested ? "gray" : color}
                    strokeWidth={1}
                    dash={[5, 5]}
                />
                {/* Two dots (circles) indicating the bisected segment */}
                <Circle x={point1.px} y={point1.py} radius={3} fill={color} />
                <Circle x={point2.px} y={point2.py} radius={3} fill={color} />
                {/* Perpendicular symbol lines */}
                {symbolLines && (
                    <>
                        <Line
                            points={symbolLines.line1}
                            stroke={isDefaultOrSuggested ? "gray" : color}
                            strokeWidth={1}
                        />
                        <Line
                            points={symbolLines.line2}
                            stroke={isDefaultOrSuggested ? "gray" : color}
                            strokeWidth={1}
                        />
                    </>
                )}
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
    const b = bisectorPoint.py - m * bisectorPoint.px;

    // Find intersection points with canvas boundaries
    // Start from the bisector point and extend in both directions
    const startX = bisectorPoint.px;
    const startY = bisectorPoint.py;

    // Calculate intersections with all four boundaries
    const intersections = [];

    // Left boundary (x = 0)
    const leftY = b;
    if (leftY >= 0 && leftY <= height) {
        intersections.push({ x: 0, y: leftY });
    }

    // Right boundary (x = width)
    const rightY = m * width + b;
    if (rightY >= 0 && rightY <= height) {
        intersections.push({ x: width, y: rightY });
    }

    // Top boundary (y = 0)
    const topX = -b / m;
    if (topX >= 0 && topX <= width) {
        intersections.push({ x: topX, y: 0 });
    }

    // Bottom boundary (y = height)
    const bottomX = (height - b) / m;
    if (bottomX >= 0 && bottomX <= width) {
        intersections.push({ x: bottomX, y: height });
    }

    // Find the two points that give the longest line segment
    let maxDistance = 0;
    let x1 = startX, y1 = startY, x2 = startX, y2 = startY;

    for (let i = 0; i < intersections.length; i++) {
        for (let j = i + 1; j < intersections.length; j++) {
            const dist = Math.sqrt(
                Math.pow(intersections[i].x - intersections[j].x, 2) +
                Math.pow(intersections[i].y - intersections[j].y, 2)
            );
            if (dist > maxDistance) {
                maxDistance = dist;
                x1 = intersections[i].x;
                y1 = intersections[i].y;
                x2 = intersections[j].x;
                y2 = intersections[j].y;
            }
        }
    }

    // If no valid intersections found, use the bisector point as both endpoints
    if (intersections.length === 0) {
        x1 = x2 = startX;
        y1 = y2 = startY;
    }

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
            {/* Solid bisector line */}
            <Line
                points={[x1, y1, x2, y2]}
                stroke={color}
                strokeWidth={2}
            />
            {/* Horizontal dotted line joining the two points */}
            <Line
                points={[point1.px, point1.py, point2.px, point2.py]}
                stroke={isDefaultOrSuggested ? "gray" : color}
                strokeWidth={1}
                dash={[5, 5]}
            />
            {/* Two dots (circles) indicating the bisected segment */}
            <Circle x={point1.px} y={point1.py} radius={3} fill={color} />
            <Circle x={point2.px} y={point2.py} radius={3} fill={color} />
            {/* Perpendicular symbol lines */}
            {symbolLines && (
                <>
                    <Line
                        points={symbolLines.line1}
                        stroke={isDefaultOrSuggested ? "gray" : color}
                        strokeWidth={1}
                    />
                    <Line
                        points={symbolLines.line2}
                        stroke={isDefaultOrSuggested ? "gray" : color}
                        strokeWidth={1}
                    />
                </>
            )}
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

export function CanvasPpToLine({ shape, getPhysicalCoords }: { shape: PpToLineShape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    if (shape.points.length === 0) return null;

    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;
    const isDefaultOrSuggested = shape.state === ShapeState.Default || shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;
    const isHinted = shape.state === ShapeState.Hinted;

    // Early return for hinted state - only show dotted line
    if (isHinted) {
        if (shape.points.length < 2) return null;

        const width = window.innerWidth;
        const height = window.innerHeight;

        // Get the two points
        const point1 = getPhysicalCoords(shape.points[0]);
        const point2 = getPhysicalCoords(shape.points[1]);

        // Calculate line direction
        const dx = point2.px - point1.px;
        const dy = point2.py - point1.py;
        const length = Math.sqrt(dx * dx + dy * dy);

        if (length === 0) return null; // Avoid division by zero

        // Calculate line equation: y = mx + b
        const m = dy / dx;
        const b = point1.py - m * point1.px;

        // Find intersection points with canvas boundaries
        const intersections = [];

        // Left boundary (x = 0)
        const leftY = b;
        if (leftY >= 0 && leftY <= height) {
            intersections.push({ x: 0, y: leftY });
        }

        // Right boundary (x = width)
        const rightY = m * width + b;
        if (rightY >= 0 && rightY <= height) {
            intersections.push({ x: width, y: rightY });
        }

        // Top boundary (y = 0)
        const topX = -b / m;
        if (topX >= 0 && topX <= width) {
            intersections.push({ x: topX, y: 0 });
        }

        // Bottom boundary (y = height)
        const bottomX = (height - b) / m;
        if (bottomX >= 0 && bottomX <= width) {
            intersections.push({ x: bottomX, y: height });
        }

        // Find the two points that give the longest line segment
        let maxDistance = 0;
        let x1 = point1.px, y1 = point1.py, x2 = point2.px, y2 = point2.py;

        for (let i = 0; i < intersections.length; i++) {
            for (let j = i + 1; j < intersections.length; j++) {
                const dist = Math.sqrt(
                    Math.pow(intersections[i].x - intersections[j].x, 2) +
                    Math.pow(intersections[i].y - intersections[j].y, 2)
                );
                if (dist > maxDistance) {
                    maxDistance = dist;
                    x1 = intersections[i].x;
                    y1 = intersections[i].y;
                    x2 = intersections[j].x;
                    y2 = intersections[j].y;
                }
            }
        }

        // If no valid intersections found, use the original points
        if (intersections.length === 0) {
            x1 = point1.px;
            y1 = point1.py;
            x2 = point2.px;
            y2 = point2.py;
        }

        return (
            <Line
                points={[x1, y1, x2, y2]}
                stroke="lightgray"
                strokeWidth={1}
                dash={[5, 5]}
            />
        );
    }

    // Get the defined line (perpendicular line)
    const definedLine = shape.getDefinedLine();
    if (!definedLine) return null;

    // Convert the defined line to physical coordinates
    const ppPoint = getPhysicalCoords(definedLine.point);
    const ppNormal = definedLine.n;

    // Calculate perpendicular symbol directions
    // The normal represents the direction of the perpendicular line
    const lineDirX = ppNormal.y; // Rotate normal by 90 degrees to get line direction
    const lineDirY = ppNormal.x;
    const lineLength = Math.sqrt(lineDirX * lineDirX + lineDirY * lineDirY);

    let lineDirNormX = 0, lineDirNormY = 0, perpDirX = 0, perpDirY = 0;
    if (lineLength > 0) {
        // Normalize the line direction
        lineDirNormX = lineDirX / lineLength;
        lineDirNormY = lineDirY / lineLength;

        // Perpendicular direction (perpendicular to the line, parallel to original line)
        perpDirX = -lineDirNormY;
        perpDirY = lineDirNormX;
    }

    const symbolLength = 10;

    // Calculate perpendicular symbol lines at the pp point
    let symbolLines = null;
    if (lineLength > 0) {
        // Calculate points for the perpendicular symbol at the pp point
        const pp = ppPoint;
        const lineOffset = lineDirNormX * symbolLength;
        const perpOffset = perpDirX * symbolLength;

        // First line: from pp + lineDir * 7 to pp + lineDir * 7 + perpDir * 7
        const line1StartX = pp.px + lineOffset;
        const line1StartY = pp.py + lineDirNormY * symbolLength;
        const line1EndX = pp.px + lineOffset + perpOffset;
        const line1EndY = pp.py + lineDirNormY * symbolLength + perpDirY * symbolLength;

        // Second line: from pp + perpDir * 7 to pp + lineDir * 7 + perpDir * 7
        const line2StartX = pp.px + perpOffset;
        const line2StartY = pp.py + perpDirY * symbolLength;
        const line2EndX = pp.px + lineOffset + perpOffset;
        const line2EndY = pp.py + lineDirNormY * symbolLength + perpDirY * symbolLength;

        symbolLines = {
            line1: [line1StartX, line1StartY, line1EndX, line1EndY],
            line2: [line2StartX, line2StartY, line2EndX, line2EndY]
        };
    }

    // Calculate the full perpendicular line that extends across the visible area
    const width = window.innerWidth;
    const height = window.innerHeight;

    // Calculate line equation for the perpendicular: y = mx + b
    // Convert normal vector to direction vector: rotate by 90 degrees
    // Note: y-axis is flipped in physical coordinates, so we use ppNormal.y directly
    const dx = ppNormal.y;
    const dy = ppNormal.x;

    if (dx === 0) {
        // Vertical perpendicular line
        const x = ppPoint.px;
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
                {/* Solid perpendicular line */}
                <Line
                    points={linePoints}
                    stroke={color}
                    strokeWidth={2}
                />
                {/* Perpendicular symbol lines */}
                {symbolLines && (
                    <>
                        <Line
                            points={symbolLines.line1}
                            stroke={isDefaultOrSuggested ? "gray" : color}
                            strokeWidth={1}
                        />
                        <Line
                            points={symbolLines.line2}
                            stroke={isDefaultOrSuggested ? "gray" : color}
                            strokeWidth={1}
                        />
                    </>
                )}
                <Text
                    x={x + 10}
                    y={ppPoint.py - 20}
                    text={shape.dbObject.name}
                    fontSize={16}
                    fill={color}
                />
            </>
        );
    }

    const m = dy / dx;
    const b = ppPoint.py - m * ppPoint.px;

    // Find intersection points with canvas boundaries
    // Start from the pp point and extend in both directions
    const startX = ppPoint.px;
    const startY = ppPoint.py;

    // Calculate intersections with all four boundaries
    const intersections = [];

    // Left boundary (x = 0)
    const leftY = b;
    if (leftY >= 0 && leftY <= height) {
        intersections.push({ x: 0, y: leftY });
    }

    // Right boundary (x = width)
    const rightY = m * width + b;
    if (rightY >= 0 && rightY <= height) {
        intersections.push({ x: width, y: rightY });
    }

    // Top boundary (y = 0)
    const topX = -b / m;
    if (topX >= 0 && topX <= width) {
        intersections.push({ x: topX, y: 0 });
    }

    // Bottom boundary (y = height)
    const bottomX = (height - b) / m;
    if (bottomX >= 0 && bottomX <= width) {
        intersections.push({ x: bottomX, y: height });
    }

    // Find the two points that give the longest line segment
    let maxDistance = 0;
    let x1 = startX, y1 = startY, x2 = startX, y2 = startY;

    for (let i = 0; i < intersections.length; i++) {
        for (let j = i + 1; j < intersections.length; j++) {
            const dist = Math.sqrt(
                Math.pow(intersections[i].x - intersections[j].x, 2) +
                Math.pow(intersections[i].y - intersections[j].y, 2)
            );
            if (dist > maxDistance) {
                maxDistance = dist;
                x1 = intersections[i].x;
                y1 = intersections[i].y;
                x2 = intersections[j].x;
                y2 = intersections[j].y;
            }
        }
    }

    // If no valid intersections found, use the pp point as both endpoints
    if (intersections.length === 0) {
        x1 = x2 = startX;
        y1 = y2 = startY;
    }

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
            {/* Solid perpendicular line */}
            <Line
                points={[x1, y1, x2, y2]}
                stroke={color}
                strokeWidth={2}
            />
            {/* Perpendicular symbol lines */}
            {symbolLines && (
                <>
                    <Line
                        points={symbolLines.line1}
                        stroke={isDefaultOrSuggested ? "gray" : color}
                        strokeWidth={1}
                    />
                    <Line
                        points={symbolLines.line2}
                        stroke={isDefaultOrSuggested ? "gray" : color}
                        strokeWidth={1}
                    />
                </>
            )}
            <Text
                x={ppPoint.px + 10}
                y={ppPoint.py - 20}
                text={shape.dbObject.name}
                fontSize={16}
                fill={color}
            />
        </>
    );
}

export function CanvasPlToLine({ shape, getPhysicalCoords }: { shape: PlToLineShape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    if (shape.points.length === 0) return null;

    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;
    const isHinted = shape.state === ShapeState.Hinted;

    // Early return for hinted state - only show dotted line
    if (isHinted) {
        if (shape.points.length < 2) return null;

        const width = window.innerWidth;
        const height = window.innerHeight;

        // Get the two points
        const point1 = getPhysicalCoords(shape.points[0]);
        const point2 = getPhysicalCoords(shape.points[1]);

        // Calculate line direction
        const dx = point2.px - point1.px;
        const dy = point2.py - point1.py;
        const length = Math.sqrt(dx * dx + dy * dy);

        if (length === 0) return null; // Avoid division by zero

        // Calculate line equation: y = mx + b
        const m = dy / dx;
        const b = point1.py - m * point1.px;

        // Find intersection points with canvas boundaries
        const intersections = [];

        // Left boundary (x = 0)
        const leftY = b;
        if (leftY >= 0 && leftY <= height) {
            intersections.push({ x: 0, y: leftY });
        }

        // Right boundary (x = width)
        const rightY = m * width + b;
        if (rightY >= 0 && rightY <= height) {
            intersections.push({ x: width, y: rightY });
        }

        // Top boundary (y = 0)
        const topX = -b / m;
        if (topX >= 0 && topX <= width) {
            intersections.push({ x: topX, y: 0 });
        }

        // Bottom boundary (y = height)
        const bottomX = (height - b) / m;
        if (bottomX >= 0 && bottomX <= width) {
            intersections.push({ x: bottomX, y: height });
        }

        // Find the two points that give the longest line segment
        let maxDistance = 0;
        let x1 = point1.px, y1 = point1.py, x2 = point2.px, y2 = point2.py;

        for (let i = 0; i < intersections.length; i++) {
            for (let j = i + 1; j < intersections.length; j++) {
                const dist = Math.sqrt(
                    Math.pow(intersections[i].x - intersections[j].x, 2) +
                    Math.pow(intersections[i].y - intersections[j].y, 2)
                );
                if (dist > maxDistance) {
                    maxDistance = dist;
                    x1 = intersections[i].x;
                    y1 = intersections[i].y;
                    x2 = intersections[j].x;
                    y2 = intersections[j].y;
                }
            }
        }

        // If no valid intersections found, use the original points
        if (intersections.length === 0) {
            x1 = point1.px;
            y1 = point1.py;
            x2 = point2.px;
            y2 = point2.py;
        }

        return (
            <Line
                points={[x1, y1, x2, y2]}
                stroke="lightgray"
                strokeWidth={1}
                dash={[5, 5]}
            />
        );
    }

    // Get the defined line (parallel line)
    const definedLine = shape.getDefinedLine();
    if (!definedLine) return null;

    // Convert the defined line to physical coordinates
    const plPoint = getPhysicalCoords(definedLine.point);
    const plNormal = definedLine.n;

    // Calculate the full parallel line that extends across the visible area
    const width = window.innerWidth;
    const height = window.innerHeight;

    // Calculate line equation for the parallel: y = mx + b
    // Convert normal vector to direction vector: rotate by 90 degrees
    // Note: y-axis is flipped in physical coordinates, so we use plNormal.y directly
    const dx = plNormal.y;
    const dy = plNormal.x;

    if (dx === 0) {
        // Vertical parallel line
        const x = plPoint.px;
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
                {/* Solid parallel line */}
                <Line
                    points={linePoints}
                    stroke={color}
                    strokeWidth={2}
                />
                <Text
                    x={x + 10}
                    y={plPoint.py - 20}
                    text={shape.dbObject.name}
                    fontSize={16}
                    fill={color}
                />
            </>
        );
    }

    const m = dy / dx;
    const b = plPoint.py - m * plPoint.px;

    // Find intersection points with canvas boundaries
    // Start from the pl point and extend in both directions
    const startX = plPoint.px;
    const startY = plPoint.py;

    // Calculate intersections with all four boundaries
    const intersections = [];

    // Left boundary (x = 0)
    const leftY = b;
    if (leftY >= 0 && leftY <= height) {
        intersections.push({ x: 0, y: leftY });
    }

    // Right boundary (x = width)
    const rightY = m * width + b;
    if (rightY >= 0 && rightY <= height) {
        intersections.push({ x: width, y: rightY });
    }

    // Top boundary (y = 0)
    const topX = -b / m;
    if (topX >= 0 && topX <= width) {
        intersections.push({ x: topX, y: 0 });
    }

    // Bottom boundary (y = height)
    const bottomX = (height - b) / m;
    if (bottomX >= 0 && bottomX <= width) {
        intersections.push({ x: bottomX, y: height });
    }

    // Find the two points that give the longest line segment
    let maxDistance = 0;
    let x1 = startX, y1 = startY, x2 = startX, y2 = startY;

    for (let i = 0; i < intersections.length; i++) {
        for (let j = i + 1; j < intersections.length; j++) {
            const dist = Math.sqrt(
                Math.pow(intersections[i].x - intersections[j].x, 2) +
                Math.pow(intersections[i].y - intersections[j].y, 2)
            );
            if (dist > maxDistance) {
                maxDistance = dist;
                x1 = intersections[i].x;
                y1 = intersections[i].y;
                x2 = intersections[j].x;
                y2 = intersections[j].y;
            }
        }
    }

    // If no valid intersections found, use the pl point as both endpoints
    if (intersections.length === 0) {
        x1 = x2 = startX;
        y1 = y2 = startY;
    }

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
            {/* Solid parallel line */}
            <Line
                points={[x1, y1, x2, y2]}
                stroke={color}
                strokeWidth={2}
            />
            <Text
                x={plPoint.px + 10}
                y={plPoint.py - 20}
                text={shape.dbObject.name}
                fontSize={16}
                fill={color}
            />
        </>
    );
}

export function CanvasLocus({ shape, getPhysicalCoords }: { shape: LocusShape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    const coords = shape.points[0];
    console.log("LocusShape", shape);
    if (!coords) return null;
    const { px, py } = getPhysicalCoords(coords);

    // Use locus ordinal for color selection
    const ordinal = shape.locusOrdinal;
    const color = shape.state === ShapeState.Hinted ? "lightgray" : PLOT_COLORS[ordinal % PLOT_COLORS.length];

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

export function CanvasSlidingPoint({ shape, getPhysicalCoords }: { shape: SlidingPointShape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    const coords = shape.points[0];
    if (!coords) return null;
    const { px, py } = getPhysicalCoords(coords);
    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;
    const isHinted = shape.state === ShapeState.Hinted;

    // Get the line direction from the shape
    const lineDirection = shape.lineDirection || { x: 1, y: 0 };

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

export function CanvasTwoPointDistanceInvariant({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    if (shape.points.length < 2) return null;

    const point1 = getPhysicalCoords(shape.points[0]);
    const point2 = getPhysicalCoords(shape.points[1]);
    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;

    // Use gray color for Default or Suggested state
    const lineColor = (shape.state === ShapeState.Default || shape.state === ShapeState.Suggested) ? 'gray' : color;

    // Calculate line direction and perpendicular direction
    const dx = point2.px - point1.px;
    const dy = point2.py - point1.py;
    const length = Math.sqrt(dx * dx + dy * dy);

    if (length === 0) return null; // Avoid division by zero

    // Normalize direction vectors
    const segX = dx / length;
    const segY = dy / length;
    const perpX = -segY; // Perpendicular direction
    const perpY = segX;

    // Parameters for the measuring arrow
    const arrowLength = 8; // Length of arrow head
    const arrowAngle = Math.PI / 6; // 30 degrees
    const perpLineLength = 6; // Length of perpendicular line segments
    const perpLineHalfLength = perpLineLength / 2;

    // Calculate arrow points for point1
    const arrow1BaseX = point1.px + segX * arrowLength;
    const arrow1BaseY = point1.py + segY * arrowLength;
    const arrow1LeftX = arrow1BaseX + segX * arrowLength * Math.cos(arrowAngle) - segY * arrowLength * Math.sin(arrowAngle);
    const arrow1LeftY = arrow1BaseY + segY * arrowLength * Math.cos(arrowAngle) + segX * arrowLength * Math.sin(arrowAngle);
    const arrow1RightX = arrow1BaseX + segX * arrowLength * Math.cos(arrowAngle) + segY * arrowLength * Math.sin(arrowAngle);
    const arrow1RightY = arrow1BaseY + segY * arrowLength * Math.cos(arrowAngle) - segX * arrowLength * Math.sin(arrowAngle);

    // Calculate arrow points for point2
    const arrow2BaseX = point2.px - segX * arrowLength;
    const arrow2BaseY = point2.py - segY * arrowLength;
    const arrow2LeftX = arrow2BaseX - segX * arrowLength * Math.cos(arrowAngle) - segY * arrowLength * Math.sin(arrowAngle);
    const arrow2LeftY = arrow2BaseY - segY * arrowLength * Math.cos(arrowAngle) + segX * arrowLength * Math.sin(arrowAngle);
    const arrow2RightX = arrow2BaseX - segX * arrowLength * Math.cos(arrowAngle) + segY * arrowLength * Math.sin(arrowAngle);
    const arrow2RightY = arrow2BaseY - segY * arrowLength * Math.cos(arrowAngle) - segX * arrowLength * Math.sin(arrowAngle);

    // Calculate perpendicular line segments at endpoints
    const perp1StartX = point1.px + perpX * perpLineHalfLength;
    const perp1StartY = point1.py + perpY * perpLineHalfLength;
    const perp1EndX = point1.px - perpX * perpLineHalfLength;
    const perp1EndY = point1.py - perpY * perpLineHalfLength;

    const perp2StartX = point2.px + perpX * perpLineHalfLength;
    const perp2StartY = point2.py + perpY * perpLineHalfLength;
    const perp2EndX = point2.px - perpX * perpLineHalfLength;
    const perp2EndY = point2.py - perpY * perpLineHalfLength;

    return (
        <>
            {/* Glow - pink background line segment */}
            {isSuggested && (
                <Line
                    points={[point1.px, point1.py, point2.px, point2.py]}
                    stroke="#ffb6c1"
                    strokeWidth={8}
                    opacity={0.4}
                />
            )}

            {/* Main dotted line */}
            <Line
                points={[point1.px, point1.py, point2.px, point2.py]}
                stroke={lineColor}
                strokeWidth={1}
                dash={[3, 3]}
            />

            {/* Perpendicular line segment at point1 */}
            <Line
                points={[perp1StartX, perp1StartY, perp1EndX, perp1EndY]}
                stroke={lineColor}
                strokeWidth={1}
            />

            {/* Perpendicular line segment at point2 */}
            <Line
                points={[perp2StartX, perp2StartY, perp2EndX, perp2EndY]}
                stroke={lineColor}
                strokeWidth={1}
            />

            {/* Arrow at point1 */}
            <Line
                points={[arrow1BaseX, arrow1BaseY, arrow1LeftX, arrow1LeftY]}
                stroke={lineColor}
                strokeWidth={1}
            />
            <Line
                points={[arrow1BaseX, arrow1BaseY, arrow1RightX, arrow1RightY]}
                stroke={lineColor}
                strokeWidth={1}
            />

            {/* Arrow at point2 */}
            <Line
                points={[arrow2BaseX, arrow2BaseY, arrow2LeftX, arrow2LeftY]}
                stroke={lineColor}
                strokeWidth={1}
            />
            <Line
                points={[arrow2BaseX, arrow2BaseY, arrow2RightX, arrow2RightY]}
                stroke={lineColor}
                strokeWidth={1}
            />
        </>
    );
}

export function CanvasPointToLineDistanceInvariant({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    if (shape.points.length < 1) return null;

    const point1 = getPhysicalCoords(shape.points[0]);
    const color = getColor(shape);
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;

    // Use gray color for Default or Suggested state
    const lineColor = (shape.state === ShapeState.Default || shape.state === ShapeState.Suggested) ? 'gray' : color;

    // If there's no second point, just show the first point
    if (shape.points.length < 2) {
        return (
            <>
                {/* Glow - pink background circle */}
                {isSuggested && (
                    <Circle
                        x={point1.px}
                        y={point1.py}
                        radius={11}
                        fill="#ffb6c1"
                        opacity={0.4}
                    />
                )}
                <Circle x={point1.px} y={point1.py} radius={3} fill={lineColor} />
            </>
        );
    }

    const point2 = getPhysicalCoords(shape.points[1]);

    // Calculate line direction and perpendicular direction
    const dx = point2.px - point1.px;
    const dy = point2.py - point1.py;
    const length = Math.sqrt(dx * dx + dy * dy);

    if (length === 0) return null; // Avoid division by zero

    // Normalize direction vectors
    const segX = dx / length;
    const segY = dy / length;
    const perpX = -segY; // Perpendicular direction
    const perpY = segX;

    // Parameters for the measuring arrow
    const arrowLength = 8; // Length of arrow head
    const arrowAngle = Math.PI / 6; // 30 degrees
    const perpLineLength = 6; // Length of perpendicular line segments
    const perpLineHalfLength = perpLineLength / 2;

    // Calculate arrow points for point1
    const arrow1BaseX = point1.px + segX * arrowLength;
    const arrow1BaseY = point1.py + segY * arrowLength;
    const arrow1LeftX = arrow1BaseX + segX * arrowLength * Math.cos(arrowAngle) - segY * arrowLength * Math.sin(arrowAngle);
    const arrow1LeftY = arrow1BaseY + segY * arrowLength * Math.cos(arrowAngle) + segX * arrowLength * Math.sin(arrowAngle);
    const arrow1RightX = arrow1BaseX + segX * arrowLength * Math.cos(arrowAngle) + segY * arrowLength * Math.sin(arrowAngle);
    const arrow1RightY = arrow1BaseY + segY * arrowLength * Math.cos(arrowAngle) - segX * arrowLength * Math.sin(arrowAngle);

    // Calculate arrow points for point2
    const arrow2BaseX = point2.px - segX * arrowLength;
    const arrow2BaseY = point2.py - segY * arrowLength;
    const arrow2LeftX = arrow2BaseX - segX * arrowLength * Math.cos(arrowAngle) - segY * arrowLength * Math.sin(arrowAngle);
    const arrow2LeftY = arrow2BaseY - segY * arrowLength * Math.cos(arrowAngle) + segX * arrowLength * Math.sin(arrowAngle);
    const arrow2RightX = arrow2BaseX - segX * arrowLength * Math.cos(arrowAngle) + segY * arrowLength * Math.sin(arrowAngle);
    const arrow2RightY = arrow2BaseY - segY * arrowLength * Math.cos(arrowAngle) - segX * arrowLength * Math.sin(arrowAngle);

    // Calculate perpendicular line segment at point1 only
    const perp1StartX = point1.px + perpX * perpLineHalfLength;
    const perp1StartY = point1.py + perpY * perpLineHalfLength;
    const perp1EndX = point1.px - perpX * perpLineHalfLength;
    const perp1EndY = point1.py - perpY * perpLineHalfLength;

    return (
        <>
            {/* Glow - pink background line segment */}
            {isSuggested && (
                <Line
                    points={[point1.px, point1.py, point2.px, point2.py]}
                    stroke="#ffb6c1"
                    strokeWidth={8}
                    opacity={0.4}
                />
            )}

            {/* Main dotted line */}
            <Line
                points={[point1.px, point1.py, point2.px, point2.py]}
                stroke={lineColor}
                strokeWidth={1}
                dash={[3, 3]}
            />

            {/* Perpendicular line segment at point1 only */}
            <Line
                points={[perp1StartX, perp1StartY, perp1EndX, perp1EndY]}
                stroke={lineColor}
                strokeWidth={1}
            />

            {/* Arrow at point1 */}
            <Line
                points={[arrow1BaseX, arrow1BaseY, arrow1LeftX, arrow1LeftY]}
                stroke={lineColor}
                strokeWidth={1}
            />
            <Line
                points={[arrow1BaseX, arrow1BaseY, arrow1RightX, arrow1RightY]}
                stroke={lineColor}
                strokeWidth={1}
            />

            {/* Arrow at point2 */}
            <Line
                points={[arrow2BaseX, arrow2BaseY, arrow2LeftX, arrow2LeftY]}
                stroke={lineColor}
                strokeWidth={1}
            />
            <Line
                points={[arrow2BaseX, arrow2BaseY, arrow2RightX, arrow2RightY]}
                stroke={lineColor}
                strokeWidth={1}
            />
        </>
    );
}

export function CanvasTwoLineAngleInvariant({ shape, getPhysicalCoords }: { shape: TwoLineAngleInvariantShape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    if (shape.points.length < 1) return null;

    const intersectionPoint = getPhysicalCoords(shape.points[0]);
    let color = getColor(shape);
    if (shape.state === ShapeState.Default || shape.state === ShapeState.Suggested) {
        color = 'gray';
    }
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;

    // Cast to access line1 and line2 properties
    if (!shape.line1Points || !shape.line2Points) return null;

    const line1Points = shape.line1Points;
    const line2Points = shape.line2Points;

    // Circle parameters
    const circleRadius = 20;
    const circleCenter = intersectionPoint;

    // Function to find intersection points of a circle with a line
    const findCircleLineIntersections = (linePoints: Vector2d[], center: { px: number; py: number }, radius: number) => {
        if (linePoints.length < 2) {
            return [];
        }

        const p1 = getPhysicalCoords(linePoints[0]);
        const p2 = getPhysicalCoords(linePoints[1]);

        // Line direction vector
        const dx = p2.px - p1.px;
        const dy = p2.py - p1.py;
        const lineLength = Math.sqrt(dx * dx + dy * dy);

        if (lineLength === 0) return [];

        // Normalize line direction
        const lineDirX = dx / lineLength;
        const lineDirY = dy / lineLength;

        // Vector from line start to circle center
        const toCenterX = center.px - p1.px;
        const toCenterY = center.py - p1.py;

        // Projection of center onto line
        const projection = toCenterX * lineDirX + toCenterY * lineDirY;
        const closestPointX = p1.px + projection * lineDirX;
        const closestPointY = p1.py + projection * lineDirY;

        // Distance from circle center to line
        const distanceToLine = Math.sqrt(
            Math.pow(center.px - closestPointX, 2) + Math.pow(center.py - closestPointY, 2)
        );

        // If line is too far from circle, no intersection
        if (distanceToLine > radius) return [];

        // If line is tangent to circle, one intersection point
        if (Math.abs(distanceToLine - radius) < 1e-6) {
            return [{ px: closestPointX, py: closestPointY }];
        }

        // Two intersection points
        const halfChord = Math.sqrt(radius * radius - distanceToLine * distanceToLine);
        const intersection1X = closestPointX + halfChord * lineDirX;
        const intersection1Y = closestPointY + halfChord * lineDirY;
        const intersection2X = closestPointX - halfChord * lineDirX;
        const intersection2Y = closestPointY - halfChord * lineDirY;

        return [
            { px: intersection1X, py: intersection1Y },
            { px: intersection2X, py: intersection2Y }
        ];
    };

    // Find intersection points with both lines
    const line1Intersections = findCircleLineIntersections(line1Points, circleCenter, circleRadius);
    const line2Intersections = findCircleLineIntersections(line2Points, circleCenter, circleRadius);

    // Combine all intersection points
    const allIntersections = [...line1Intersections, ...line2Intersections];

    if (allIntersections.length < 2) {
        return null;
    }

    // Find the two closest points
    let minDistance = Infinity;
    let closestPair = [allIntersections[0], allIntersections[1]];

    for (let i = 0; i < allIntersections.length; i++) {
        for (let j = i + 1; j < allIntersections.length; j++) {
            const p1 = allIntersections[i];
            const p2 = allIntersections[j];
            const distance = Math.sqrt(
                Math.pow(p1.px - p2.px, 2) + Math.pow(p1.py - p2.py, 2)
            );
            if (distance < minDistance) {
                minDistance = distance;
                closestPair = [p1, p2];
            }
        }
    }

    // Calculate arc parameters
    const startPoint = closestPair[0];
    const endPoint = closestPair[1];

    // Calculate angles for the arc
    const startAngle = Math.atan2(startPoint.py - circleCenter.py, startPoint.px - circleCenter.px);
    const endAngle = Math.atan2(endPoint.py - circleCenter.py, endPoint.px - circleCenter.px);

    // Ensure we draw the shorter arc
    let angleDiff = endAngle - startAngle;
    if (angleDiff > Math.PI) {
        angleDiff -= 2 * Math.PI;
    } else if (angleDiff < -Math.PI) {
        angleDiff += 2 * Math.PI;
    }

    // Create SVG path for the arc
    const largeArcFlag = Math.abs(angleDiff) > Math.PI ? 1 : 0;
    const sweepFlag = angleDiff > 0 ? 1 : 0;

    const pathData = [
        `M ${startPoint.px} ${startPoint.py}`,
        `A ${circleRadius} ${circleRadius} 0 ${largeArcFlag} ${sweepFlag} ${endPoint.px} ${endPoint.py}`
    ].join(' ');

    return (
        <>
            {/* Glow - pink background circle */}
            {isSuggested && (
                <Circle
                    x={intersectionPoint.px}
                    y={intersectionPoint.py}
                    radius={11}
                    fill="#ffb6c1"
                    opacity={0.4}
                />
            )}

            {/* Arc connecting the two closest intersection points */}
            <Path
                data={pathData}
                stroke={color}
                strokeWidth={2}
                dash={[5, 5]}
                fill="transparent"
            />
        </>
    );
}

export function CanvasProjection({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    if (shape.points.length < 2) return null;
    const p0 = getPhysicalCoords(shape.points[0]);
    const p1 = getPhysicalCoords(shape.points[1]);

    if (shape.state === ShapeState.Hinted) {
        // Only draw a dotted line from p0 to p1
        return (
            <Line
                points={[p0.px, p0.py, p1.px, p1.py]}
                stroke="lightgray"
                strokeWidth={1}
                dash={[3, 3]}
            />
        );
    }

    // Glow for Suggested or SuggestedSelected
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;
    const color = getColor(shape);
    let lineColor = color;
    if (shape.state === ShapeState.Default || shape.state === ShapeState.Suggested) {
        lineColor = 'gray';
    }

    // Vector from p0 to p1
    const dx = p0.px - p1.px;
    const dy = p0.py - p1.py;
    const len = Math.sqrt(dx * dx + dy * dy);
    if (len === 0) return null;
    // Unit vector in direction p0->p1, scaled to 10
    const ux = (dx / len) * 10;
    const uy = (dy / len) * 10;
    // Perpendicular vector (rotated 90deg)
    const vx = -uy;
    const vy = ux;

    // Perpendicular symbol points
    // Start at p1 + (ux, uy)
    const baseX = p1.px + ux;
    const baseY = p1.py + uy;
    // First line: from base to base + (vx, vy)
    const perp1X = baseX + vx;
    const perp1Y = baseY + vy;
    // Second line: from base + (vx, vy) to p1 + (vx, vy)
    const endX = p1.px + vx;
    const endY = p1.py + vy;

    return (
        <>
            {isSuggested && (
                <Circle
                    x={p1.px}
                    y={p1.py}
                    radius={11}
                    fill="#ffb6c1"
                    opacity={0.4}
                />
            )}
            {/* Dotted line from p0 to p1 */}
            <Line
                points={[p0.px, p0.py, p1.px, p1.py]}
                stroke={lineColor}
                strokeWidth={1}
                dash={[3, 3]}
            />
            {/* Dot at p1 */}
            <Circle x={p1.px} y={p1.py} radius={3} fill={color} />
            {/* Dot at p0 */}
            <Circle x={p0.px} y={p0.py} radius={3} stroke={lineColor} strokeWidth={1} />
            {/* Perpendicular symbol */}
            <Line
                points={[baseX, baseY, perp1X, perp1Y]}
                stroke={lineColor}
                strokeWidth={1}
            />
            <Line
                points={[perp1X, perp1Y, endX, endY]}
                stroke={lineColor}
                strokeWidth={1}
            />
            {/* Label */}
            <Text x={p1.px + 10} y={p1.py - 25} text={shape.dbObject.name} fontSize={16} fill={color} />
        </>
    );
}

export function CanvasReflection({ shape, getPhysicalCoords }: { shape: Shape; getPhysicalCoords: (coords: Vector2d) => { px: number; py: number } }): React.ReactElement | null {
    if (shape.points.length < 2) return null;
    const p0 = getPhysicalCoords(shape.points[0]);
    const p1 = getPhysicalCoords(shape.points[1]);

    if (shape.state === ShapeState.Hinted) {
        // Only draw a dotted line from p0 to p1
        return (
            <Line
                points={[p0.px, p0.py, p1.px, p1.py]}
                stroke="lightgray"
                strokeWidth={1}
                dash={[3, 3]}
            />
        );
    }

    // Glow for Suggested or SuggestedSelected
    const isSuggested = shape.state === ShapeState.Suggested || shape.state === ShapeState.SuggestedSelected;
    const color = getColor(shape);
    let lineColor = color;
    if (shape.state === ShapeState.Default || shape.state === ShapeState.Suggested) {
        lineColor = 'gray';
    }

    // Calculate midpoint between p0 and p1 (intersection point with reflection line)
    const midX = (p0.px + p1.px) / 2;
    const midY = (p0.py + p1.py) / 2;

    // Vector from p0 to p1
    const dx = p0.px - p1.px;
    const dy = p0.py - p1.py;
    const len = Math.sqrt(dx * dx + dy * dy);
    if (len === 0) return null;
    // Unit vector in direction p0->p1, scaled to 10
    const ux = (dx / len) * 10;
    const uy = (dy / len) * 10;
    // Perpendicular vector (rotated 90deg)
    const vx = -uy;
    const vy = ux;

    // Perpendicular symbol points - use midpoint instead of p1
    // Start at mid + (ux, uy)
    const baseX = midX + ux;
    const baseY = midY + uy;
    // First line: from base to base + (vx, vy)
    const perp1X = baseX + vx;
    const perp1Y = baseY + vy;
    // Second line: from base + (vx, vy) to mid + (vx, vy)
    const endX = midX + vx;
    const endY = midY + vy;

    return (
        <>
            {isSuggested && (
                <Circle
                    x={p1.px}
                    y={p1.py}
                    radius={11}
                    fill="#ffb6c1"
                    opacity={0.4}
                />
            )}
            {/* Dotted line from p0 to p1 */}
            <Line
                points={[p0.px, p0.py, p1.px, p1.py]}
                stroke={lineColor}
                strokeWidth={1}
                dash={[3, 3]}
            />
            {/* Dot at p1 (reflected point) */}
            <Circle x={p1.px} y={p1.py} radius={3} fill={color} />
            {/* Dot at p0 (original point) */}
            <Circle x={p0.px} y={p0.py} radius={3} stroke={lineColor} strokeWidth={1} />
            {/* Perpendicular symbol at midpoint */}
            <Line
                points={[baseX, baseY, perp1X, perp1Y]}
                stroke={lineColor}
                strokeWidth={1}
            />
            <Line
                points={[perp1X, perp1Y, endX, endY]}
                stroke={lineColor}
                strokeWidth={1}
            />
            {/* Label */}
            <Text x={p1.px + 10} y={p1.py - 25} text={shape.dbObject.name} fontSize={16} fill={color} />
        </>
    );
}
