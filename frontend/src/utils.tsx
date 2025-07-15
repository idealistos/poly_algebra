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
import type { PartialDBObject, Shape } from "./types";
import Color from "color";

// Plot colors for locus objects (10 colors for different locus ordinals)
export const PLOT_COLORS = [
    '#d32f2f', // Red
    '#1976d2', // Blue
    '#388e3c', // Green
    '#f57c00', // Orange
    '#7b1fa2', // Purple
    '#c2185b', // Pink
    '#0097a7', // Cyan
    '#ff8f00', // Amber
    '#6d4c41', // Brown
    '#5d4037', // Dark Brown
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
    const transformedColor = Color.hsl(targetHsl.h, redHsl.s, redHsl.l);

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

export function createShapeForDBObject(dbObject: PartialDBObject, shapes: Shape[], currentActionStep?: number): Shape {
    switch (dbObject.object_type) {
        case ObjectType.FixedPoint:
            return new FixedPointShape(dbObject);
        case ObjectType.FreePoint:
            return new FreePointShape(dbObject);
        case ObjectType.Midpoint:
            if (currentActionStep === 0) {
                return new InitialPointShape(dbObject);
            } else {
                return new MidpointShape(dbObject, shapes);
            }
        case ObjectType.IntersectionPoint:
            return new IntersectionPointShape(dbObject, shapes);
        case ObjectType.SlidingPoint:
            return new SlidingPointShape(dbObject, shapes);
        case ObjectType.LineAB:
            if (currentActionStep === 0) {
                return new InitialPointShape(dbObject);
            } else {
                return new LineABShape(dbObject, shapes);
            }
        case ObjectType.Parameter:
            return new ParameterShape(dbObject);
        case ObjectType.Invariant:
            return new InvariantShape(dbObject);
        case ObjectType.Locus:
            return new LocusShape(dbObject, shapes);
        default:
            {
                const exhaustiveCheck: never = dbObject.object_type;
                throw new Error(`Unhandled object type: ${exhaustiveCheck}`);
            }
    }
}