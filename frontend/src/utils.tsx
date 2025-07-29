import { ObjectType } from "./enums";
import { FixedPointShape } from "./shapes/FixedPointShape";
import { FreePointShape } from "./shapes/FreePointShape";
import { InitialPointShape } from "./shapes/InitialPointShape";
import { InvariantShape } from "./shapes/InvariantShape";
import { LineABShape } from "./shapes/LineABShape";
import { LocusShape } from "./shapes/LocusShape";
import { MidpointShape } from "./shapes/MidpointShape";
import { IntersectionPointShape } from "./shapes/IntersectionPointShape";
import { SlidingPointShape } from "./shapes/SlidingPointShape";
import { ParameterShape } from "./shapes/ParameterShape";
import { TwoPointDistanceInvariantShape } from "./shapes/TwoPointDistanceInvariantShape";
import { PointToLineDistanceInvariantShape } from "./shapes/PointToLineDistanceInvariantShape";
import type { Action, ObjectProperties, PartialDBObject, Shape } from "./types";
import type { Vector2d } from 'konva/lib/types';
import Color from "color";
import { TwoLineAngleInvariantShape } from "./shapes/TwoLineAngleInvariantShape";
import { PpBisectorShape } from "./shapes/PpBisectorShape";
import type { LineBasedShape } from "./shapes/LineBasedShape";
import { PpToLineShape } from "./shapes/PpToLineShape";
import { ReflectionShape } from "./shapes/ReflectionShape";
import { ProjectionShape } from "./shapes/ProjectionShape";
import { PlToLineShape } from "./shapes/PlToLineShape";
import { ComputedPointShape } from './shapes/ComputedPointShape';
import { ScaledVectorPointShape } from "./shapes/ScaledVectorPointShape";

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

export function getPointDescription(point: string | null): string {
    if (point === null) {
        return "?";
    }
    if (point.includes(",")) {
        return `(${point})`;
    }
    return point;
}

export function getActionTitle(action: Action): string {
    return action.description.split(":")[0];
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

export function createShapeForDBObject(dbObject: PartialDBObject, shapes: Shape[], currentActionStep?: number): Shape {
    switch (dbObject.object_type) {
        case ObjectType.FixedPoint:
            return new FixedPointShape(dbObject);
        case ObjectType.FreePoint:
            return new FreePointShape(dbObject);
        case ObjectType.Midpoint:
            if (currentActionStep === 0) {
                return new InitialPointShape(dbObject, shapes);
            } else {
                return new MidpointShape(dbObject, shapes);
            }
        case ObjectType.IntersectionPoint:
            return new IntersectionPointShape(dbObject, shapes);
        case ObjectType.SlidingPoint:
            return new SlidingPointShape(dbObject, shapes);
        case ObjectType.Projection:
            if (currentActionStep === 0) {
                return new InitialPointShape(dbObject, shapes);
            } else {
                return new ProjectionShape(dbObject, shapes);
            }
        case ObjectType.Reflection:
            if (currentActionStep === 0) {
                return new InitialPointShape(dbObject, shapes);
            } else {
                return new ReflectionShape(dbObject, shapes);
            }
        case ObjectType.ScaledVectorPoint:
            if (currentActionStep === 1) {
                return new InitialPointShape(dbObject, shapes);
            } else {
                return new ScaledVectorPointShape(dbObject, shapes);
            }
        case ObjectType.ComputedPoint:
            return new ComputedPointShape(dbObject);
        case ObjectType.LineAB:
            if (currentActionStep === 0) {
                return new InitialPointShape(dbObject, shapes);
            } else {
                return new LineABShape(dbObject, shapes);
            }
        case ObjectType.PpBisector:
            if (currentActionStep === 0) {
                return new InitialPointShape(dbObject, shapes);
            } else {
                return new PpBisectorShape(dbObject, shapes);
            }
        case ObjectType.PpToLine:
            if (currentActionStep === 0) {
                return new InitialPointShape(dbObject, shapes);
            } else {
                return new PpToLineShape(dbObject, shapes);
            }
        case ObjectType.PlToLine:
            if (currentActionStep === 0) {
                return new InitialPointShape(dbObject, shapes);
            } else {
                return new PlToLineShape(dbObject, shapes);
            }
        case ObjectType.Parameter:
            return new ParameterShape(dbObject);
        case ObjectType.Invariant:
            return new InvariantShape(dbObject);
        case ObjectType.Locus:
            return new LocusShape(dbObject, shapes);
        case ObjectType.TwoPointDistanceInvariant:
            if (currentActionStep === 0) {
                return new InitialPointShape(dbObject, shapes);
            } else {
                return new TwoPointDistanceInvariantShape(dbObject, shapes);
            }
        case ObjectType.PointToLineDistanceInvariant:
            if (currentActionStep === 0) {
                return new InitialPointShape(dbObject, shapes);
            } else {
                return new PointToLineDistanceInvariantShape(dbObject, shapes);
            }
        case ObjectType.TwoLineAngleInvariant:
            if (dbObject.properties && "line1" in dbObject.properties && "line2" in dbObject.properties) {
                return new TwoLineAngleInvariantShape(dbObject, shapes);
            } else {
                return new InitialPointShape(dbObject, shapes);
            }
        default:
            {
                const exhaustiveCheck: never = dbObject.object_type;
                throw new Error(`Unhandled object type: ${exhaustiveCheck}`);
            }
    }
}

export function getDBObjectForExpressions(name: string, expressions: string[], action: Action): PartialDBObject {
    const objectType = action.object_types[0];
    switch (objectType) {
        case ObjectType.ScaledVectorPoint:
            return {
                name,
                object_type: ObjectType.ScaledVectorPoint,
                properties: { k: expressions[0] },
            };
        case ObjectType.ComputedPoint:
            return {
                name,
                object_type: ObjectType.ComputedPoint,
                properties: { x_expr: expressions[0], y_expr: expressions[1] },
            };
        case ObjectType.Invariant:
            return {
                name,
                object_type: ObjectType.Invariant,
                properties: { formula: expressions[0] },
            };
        default:
            throw new Error(`Object type isn't defined with expressions: ${objectType}`);
    }
}

export function getDBPropertiesForLine(shape: LineBasedShape, objectType: ObjectType, actionStep: number): Partial<ObjectProperties> {
    switch (objectType) {
        case ObjectType.PpToLine:
        case ObjectType.PlToLine:
        case ObjectType.PointToLineDistanceInvariant:
        case ObjectType.Projection:
        case ObjectType.Reflection:
        case ObjectType.ComputedPoint:
            return {
                line: shape.dbObject.name,
            };
        case ObjectType.TwoLineAngleInvariant:
            if (actionStep === 0) {
                return { line1: shape.dbObject.name };
            } else {
                return { line2: shape.dbObject.name };
            }
        default:
            throw new Error(`Unhandled object type: ${objectType}`);
    }
}

export function checkLineAlreadyChosen(dbObject: PartialDBObject, shape: LineBasedShape): boolean {
    if (dbObject.object_type === ObjectType.TwoLineAngleInvariant) {
        return dbObject.properties != null && "line1" in dbObject.properties && dbObject.properties.line1 === shape.dbObject.name;
    }
    return false;
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
    const shape = shapes.find(s => s.dbObject.name === pointValue);
    if (shape && shape.points.length > 0) {
        return shape.getDefinedPoint()!;
    }

    return null;
}