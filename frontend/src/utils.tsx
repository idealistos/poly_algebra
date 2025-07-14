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