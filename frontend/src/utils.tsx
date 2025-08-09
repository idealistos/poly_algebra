import { ArgumentType, MOBILE_POINT_OBJECT_TYPES, ObjectType } from "./enums";
import { FixedPointShapeCreator } from "./shapes/FixedPointShape";
import { FreePointShapeCreator } from "./shapes/FreePointShape";
import { LineABShapeCreator } from "./shapes/LineABShape";
import { LocusShapeCreator } from "./shapes/LocusShape";
import { MidpointShapeCreator } from "./shapes/MidpointShape";
import { SlidingPointShapeCreator } from "./shapes/SlidingPointShape";
import { ParameterShapeCreator } from "./shapes/ParameterShape";
import { TwoPointDistanceInvariantShapeCreator } from "./shapes/TwoPointDistanceInvariantShape";
import type { Line, Shape, ShapeCreator, ShapeCreatorInput } from "./types";
import type { Vector2d } from 'konva/lib/types';
import Color from "color";
import { LineBasedShape } from "./shapes/LineBasedShape";
import { ReflectionShapeCreator } from "./shapes/ReflectionShape";
import { ProjectionShapeCreator } from "./shapes/ProjectionShape";
import { ComputedPointShapeCreator } from './shapes/ComputedPointShape';
import { BaseShape } from "./shapes/BaseShape";
import { InvariantShapeCreator } from "./shapes/InvariantShape";
import { PpBisectorShapeCreator } from "./shapes/PpBisectorShape";
import { PpToLineShapeCreator } from "./shapes/PpToLineShape";
import { PlToLineShapeCreator } from "./shapes/PlToLineShape";
import { PointToLineDistanceInvariantShapeCreator } from "./shapes/PointToLineDistanceInvariantShape";
import { IntersectionPointShapeCreator } from "./shapes/IntersectionPointShape";
import { ScaledVectorPointShapeCreator } from "./shapes/ScaledVectorPointShape";
import { TwoLineAngleInvariantShapeCreator } from "./shapes/TwoLineAngleInvariantShape";

// Plot colors for locus objects (10 colors for different locus ordinals)
export const PLOT_COLORS = [
    "#d32f2f",  // Red
    "#1976d2",  // Blue
    "#388e3c",  // Green
    "#f57c00",  // Orange
    "#7b1fa2",  // Purple
    "#c2185b",  // Pink
    "#0097a7",  // Cyan
    "#ff8f00",  // Amber
    "#6d4c41",  // Brown
    "#5d4037",  // Dark Brown
];

// Transform red interpolated color to use target hue while preserving saturation and lightness
export function transformPlotColor(redColor: { r: number; g: number; b: number }, targetHexColor: string): { r: number; g: number; b: number } {
    // Convert the red interpolated color to HSL
    const redColorObj = Color.rgb(redColor.r, redColor.g, redColor.b);
    const redHsl = redColorObj.hsl().object();

    // Convert target hex color to HSL to get the hue
    const targetColorObj = Color(targetHexColor);
    const targetHsl = targetColorObj.hsl().object();

    // Create new color with target hue but red color's saturation and lightness
    const transformedColor = Color.hsl(targetHsl.h, targetHsl.s, redHsl.l);

    // Convert back to RGB
    const rgb = transformedColor.rgb().object();
    return {
        r: Math.round(rgb.r),
        g: Math.round(rgb.g),
        b: Math.round(rgb.b)
    };
}


export function distanceToLineSegment(point: Vector2d, segmentStart: Vector2d, segmentEnd: Vector2d): number {
    // Calculate the distance from point to line segment
    const A = point.x - segmentStart.x;
    const B = point.y - segmentStart.y;
    const C = segmentEnd.x - segmentStart.x;
    const D = segmentEnd.y - segmentStart.y;

    const lenSq = C * C + D * D;

    if (lenSq === 0) {
        // Line segment is actually a point
        return Math.sqrt(A * A + B * B);
    }

    // Calculate the parameter t for the closest point on the line segment
    // t = ((point - segmentStart) · (segmentEnd - segmentStart)) / |segmentEnd - segmentStart|²
    const t = (A * C + B * D) / lenSq;

    // Clamp t to [0, 1] to ensure we're on the line segment
    const clampedT = Math.max(0, Math.min(1, t));

    // Calculate the closest point on the line segment
    const closestX = segmentStart.x + clampedT * C;
    const closestY = segmentStart.y + clampedT * D;

    // Return the distance from the given point to the closest point on the segment
    return Math.sqrt(
        Math.pow(point.x - closestX, 2) + Math.pow(point.y - closestY, 2)
    );
}



export function parsePoint(pointValue: string, shapes: Shape[]): { x: number; y: number } | null {
    // Check if it's a coordinate string like "x,y"
    const coordMatch = pointValue.match(/^(-?\d+),\s*(-?\d+)$/);
    if (coordMatch) {
        return {
            x: parseInt(coordMatch[1]),
            y: parseInt(coordMatch[2])
        };
    }

    // Otherwise, look up the object by name in shapes
    const shape = shapes.find(s => s.name === pointValue);
    if (shape && shape.getCoveredPoints().length > 0) {
        return shape.getDefinedPoint()!;
    }

    return null;
}

export function getDefinedOrGridPoint(pointValue: string, shapes: Shape[]): Shape | Vector2d | null {
    // Check if it's a coordinate string like "x,y"
    const coordMatch = pointValue.match(/^(-?\d+),\s*(-?\d+)$/);
    if (coordMatch) {
        return {
            x: parseInt(coordMatch[1]),
            y: parseInt(coordMatch[2])
        };
    }

    // Otherwise, look up the object by name in shapes
    return shapes.find(s => s.name === pointValue) ?? null;
}

export function getPointsFromInput(input: ShapeCreatorInput): Vector2d[] {
    const points = [];
    const firstIndex = input.validatedExpressions.length;
    for (const [index, arg] of input.argumentValues.entries()) {
        if (index < firstIndex) {
            continue;
        }
        if (arg == null) {
            break;
        }
        if (arg[0] instanceof BaseShape) {
            const definedPoint = arg[0].getDefinedPoint();
            if (definedPoint != null) {
                points.push(definedPoint);
            }
        } else {
            points.push(arg[0] as Vector2d);
        }
    }
    if (input.hintedObjectPoint != null) {
        points.push(input.hintedObjectPoint);
    }
    return points;
}

export function getGridOrHintedPointFromInput(input: ShapeCreatorInput): Vector2d | null {
    if (input.argumentValues[0] == null && input.hintedObjectPoint != null) {
        return input.hintedObjectPoint!;
    } else if (input.argumentValues[0] != null && !(input.argumentValues[0][0] instanceof BaseShape)) {
        return input.argumentValues[0][0] as Vector2d;
    } else {
        return null;
    }
}

export function getShapeNameOrPoint(argumentValue: Vector2d | Shape | null): string {
    if (argumentValue == null) {
        throw new Error("Argument value is null");
    }
    if (argumentValue instanceof BaseShape) {
        return argumentValue.name;
    } else {
        const point = argumentValue as Vector2d;
        return `${point.x},${point.y}`;
    }
}

export function getTwoClosestLines(
    shapes: Shape[],
    logicalPoint: Vector2d
): { shape: LineBasedShape; distance: number }[] | null {
    const lineShapes = shapes.filter(shape => shape instanceof LineBasedShape) as LineBasedShape[];

    if (lineShapes.length < 2) {
        return null;
    }

    // Calculate distances to all lines using the distanceToPoint method
    const linesWithDistances = lineShapes.map(line => ({
        shape: line,
        distance: line.distanceToPoint(logicalPoint)
    }));

    // Sort by distance and return the two closest
    linesWithDistances.sort((a, b) => a.distance - b.distance);

    return linesWithDistances.slice(0, 2);
}

export function getClosestLine(
    shapes: Shape[],
    logicalPoint: Vector2d
): { shape: LineBasedShape; distance: number } | null {
    const lineShapes = shapes.filter(shape => shape instanceof LineBasedShape) as LineBasedShape[];

    if (lineShapes.length === 0) {
        console.log("No lines found");
        return null;
    }

    // Calculate distances to all lines using the distanceToPoint method
    const linesWithDistances = lineShapes.map(line => ({
        shape: line,
        distance: line.distanceToPoint(logicalPoint)
    }));
    console.log(linesWithDistances);

    // Sort by distance and return the closest
    linesWithDistances.sort((a, b) => a.distance - b.distance);

    return linesWithDistances[0];
}

export function getClosestDefinedPoint(
    objectTypes: ObjectType[],
    shapes: Shape[],
    logicalPoint: Vector2d,
    isOccupied: (point: Vector2d) => boolean
): { shape: Shape | null; minDist: number } {
    let minDist = Infinity;
    let closest: Shape | null = null;
    for (const shape of shapes) {
        if (objectTypes.includes(shape.objectType!)) {
            const definedPoint = shape.getDefinedPoint();
            if (definedPoint) {
                const dist = Math.sqrt(
                    Math.pow(logicalPoint.x - definedPoint.x, 2) + Math.pow(logicalPoint.y - definedPoint.y, 2)
                );
                if (dist < minDist && !isOccupied(definedPoint)) {
                    minDist = dist;
                    closest = shape;
                }
            }
        }
    }
    return { shape: closest, minDist };
}

export function intersectLines(line1: Line, line2: Line): Vector2d | null {
    // Solve the system of equations:
    // n1 · (p - point1) = 0
    // n2 · (p - point2) = 0

    // This gives us:
    // n1x * (px - point1x) + n1y * (py - point1y) = 0
    // n2x * (px - point2x) + n2y * (py - point2y) = 0

    // Rearranging:
    // n1x * px + n1y * py = n1x * point1x + n1y * point1y
    // n2x * px + n2y * py = n2x * point2x + n2y * point2y

    const a11 = line1.n.x;
    const a12 = line1.n.y;
    const a21 = line2.n.x;
    const a22 = line2.n.y;

    const b1 = line1.n.x * line1.point.x + line1.n.y * line1.point.y;
    const b2 = line2.n.x * line2.point.x + line2.n.y * line2.point.y;

    // Calculate determinant
    const det = a11 * a22 - a12 * a21;

    if (Math.abs(det) < 1e-10) {
        // Lines are parallel
        return null;
    }

    // Solve using Cramer's rule
    const px = (b1 * a22 - b2 * a12) / det;
    const py = (a11 * b2 - a21 * b1) / det;

    return { x: px, y: py };
}

export function getTwoPointsOnLine(line: Line): Vector2d[] {
    return [line.point, { x: line.point.x + line.n.y, y: line.point.y - line.n.x }];
}

export function getMatchingObjectTypes(argType: ArgumentType): ObjectType[] {
    switch (argType) {
        case ArgumentType.MobilePoint:
            return MOBILE_POINT_OBJECT_TYPES;
        case ArgumentType.AnyDefinedPoint:
        case ArgumentType.AnyDefinedOrGridPoint:
            return [...MOBILE_POINT_OBJECT_TYPES, ObjectType.FixedPoint];
        case ArgumentType.IntersectionPoint:
            return [ObjectType.LineAB, ObjectType.PpBisector];
        case ArgumentType.SlidingPoint:
            return [...MOBILE_POINT_OBJECT_TYPES, ObjectType.FixedPoint];
        case ArgumentType.Line:
            return [ObjectType.LineAB, ObjectType.PpBisector];
        case ArgumentType.GridPoint:
            return [];
        default:
            {
                const exhaustiveCheck: never = argType;
                throw new Error(`Unhandled argument type: ${exhaustiveCheck}`);
            }
    }
}

export function getShapeCreator(objectType: ObjectType): ShapeCreator {
    switch (objectType) {
        case ObjectType.FixedPoint:
            return new FixedPointShapeCreator();
        case ObjectType.FreePoint:
            return new FreePointShapeCreator();
        case ObjectType.Midpoint:
            return new MidpointShapeCreator();
        case ObjectType.Projection:
            return new ProjectionShapeCreator();
        case ObjectType.Reflection:
            return new ReflectionShapeCreator();
        case ObjectType.LineAB:
            return new LineABShapeCreator();
        case ObjectType.SlidingPoint:
            return new SlidingPointShapeCreator();
        case ObjectType.Locus:
            return new LocusShapeCreator();
        case ObjectType.Parameter:
            return new ParameterShapeCreator();
        case ObjectType.TwoPointDistanceInvariant:
            return new TwoPointDistanceInvariantShapeCreator();
        case ObjectType.Invariant:
            return new InvariantShapeCreator();
        case ObjectType.ComputedPoint:
            return new ComputedPointShapeCreator();
        case ObjectType.PpBisector:
            return new PpBisectorShapeCreator();
        case ObjectType.PpToLine:
            return new PpToLineShapeCreator();
        case ObjectType.PlToLine:
            return new PlToLineShapeCreator();
        case ObjectType.PointToLineDistanceInvariant:
            return new PointToLineDistanceInvariantShapeCreator();
        case ObjectType.IntersectionPoint:
            return new IntersectionPointShapeCreator();
        case ObjectType.ScaledVectorPoint:
            return new ScaledVectorPointShapeCreator();
        case ObjectType.TwoLineAngleInvariant:
            return new TwoLineAngleInvariantShapeCreator();
        default:
            throw new Error(`Unhandled object type: ${objectType}`);
    }
}
