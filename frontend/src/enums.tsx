import type { Vector2d } from 'konva/lib/types';
import type { ObjectProperties, PartialDBObject, Shape } from './types';
import { LineABShape } from './shapes/LineABShape';
import { MidpointShape } from './shapes/MidpointShape';
import { ScaledVectorPointShape } from './shapes/ScaledVectorPointShape';

export enum ObjectType {
    FixedPoint = 'FixedPoint',
    FreePoint = 'FreePoint',
    Midpoint = 'Midpoint',
    IntersectionPoint = 'IntersectionPoint',
    SlidingPoint = 'SlidingPoint',
    Projection = 'Projection',
    Reflection = 'Reflection',
    ScaledVectorPoint = 'ScaledVectorPoint',
    ComputedPoint = 'ComputedPoint',
    LineAB = 'LineAB',
    PpBisector = 'PpBisector',
    PpToLine = 'PpToLine',
    PlToLine = 'PlToLine',
    Parameter = 'Parameter',
    TwoPointDistanceInvariant = 'TwoPointDistanceInvariant',
    PointToLineDistanceInvariant = 'PointToLineDistanceInvariant',
    TwoLineAngleInvariant = 'TwoLineAngleInvariant',
    Invariant = 'Invariant',
    Locus = 'Locus',
}

export const MOBILE_POINT_OBJECT_TYPES: ObjectType[] = [
    ObjectType.FreePoint,
    ObjectType.Midpoint,
    ObjectType.IntersectionPoint,
    ObjectType.SlidingPoint,
    ObjectType.Projection,
    ObjectType.Reflection,
    ObjectType.ScaledVectorPoint,
    ObjectType.ComputedPoint,
];

export enum ShapeState {
    Default = 'Default',
    Selected = 'Selected',
    Suggested = 'Suggested',
    SuggestedSelected = 'SuggestedSelected',
    Hinted = 'Hinted',
    BeingAdded = 'BeingAdded',
}

export enum ActionType {
    FixedPoint = 'FixedPoint',
    FreePoint = 'FreePoint',
    Midpoint = 'Midpoint',
    IntersectionPoint = 'IntersectionPoint',
    SlidingPoint = 'SlidingPoint',
    Projection = 'Projection',
    Reflection = 'Reflection',
    ScaledVectorPoint = 'ScaledVectorPoint',
    ComputedPoint = 'ComputedPoint',
    LineAB = 'LineAB',
    PpBisector = 'PpBisector',
    PpToLine = 'PpToLine',
    PlToLine = 'PlToLine',
    Parameter = 'Parameter',
    DistanceInvariant = 'DistanceInvariant',
    AngleInvariant = 'AngleInvariant',
    Invariant = 'Invariant',
    Locus = 'Locus',
}

export enum ActionGroup {
    Points = 'Points',
    Lines = 'Lines',
    Parameters = 'Parameters',
    Constraints = 'Constraints',
    Locus = 'Locus',
}

export enum ArgumentType {
    GridPoint = 'GridPoint',
    MobilePoint = 'MobilePoint',
    AnyDefinedPoint = 'AnyDefinedPoint',
    AnyDefinedOrGridPoint = 'AnyDefinedOrGridPoint',
    IntersectionPoint = 'IntersectionPoint',
    SlidingPoint = 'SlidingPoint',
    Line = 'Line',
}

export function getColor(shape: Shape) {
    switch (shape.state) {
        case ShapeState.Default:
        case ShapeState.Suggested:
            return 'black';
        case ShapeState.Selected:
        case ShapeState.SuggestedSelected:
            return 'blue';
        case ShapeState.Hinted:
            return 'lightgray';
        case ShapeState.BeingAdded:
            return 'darkgreen';
        default:
            {
                const exhaustiveCheck: never = shape.state;
                throw new Error(`Unhandled color case: ${exhaustiveCheck}`);
            }
    }
}

export function getPointDBProperties(objectType: ObjectType, lastPoint: Vector2d | string, currentActionStep: number): Partial<ObjectProperties> {
    let value;
    if (typeof lastPoint === 'string') {
        value = lastPoint;
    } else {
        value = lastPoint.x + "," + lastPoint.y;
    }
    switch (objectType) {
        case ObjectType.Midpoint:
        case ObjectType.LineAB:
        case ObjectType.PpBisector:
        case ObjectType.TwoPointDistanceInvariant:
            return currentActionStep === 0 ? { point1: value } : { point2: value };
        case ObjectType.ScaledVectorPoint:
            return currentActionStep === 1 ? { point1: value } : { point2: value };
        case ObjectType.Locus:
        case ObjectType.PointToLineDistanceInvariant:
        case ObjectType.PpToLine:
        case ObjectType.PlToLine:
        case ObjectType.Projection:
        case ObjectType.Reflection:
            return { point: value };
        case ObjectType.TwoLineAngleInvariant:
            return currentActionStep === 0 ? { line1: value } : { line2: value };
        default:
            return { value: value };
    }
}

export function getOccupiedPoints(dbObject: PartialDBObject): Vector2d[] {
    switch (dbObject.object_type) {
        case ObjectType.Midpoint:
            return new MidpointShape(dbObject, []).points;
        case ObjectType.ScaledVectorPoint:
            return new ScaledVectorPointShape(dbObject, []).points;
        case ObjectType.LineAB:
            return new LineABShape(dbObject, []).points;
        default:
            return [];
    }
}