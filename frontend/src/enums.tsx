import type { Vector2d } from 'konva/lib/types';
import type { ObjectProperties, PartialDBObject, Shape } from './types';
import { LineABShape } from './shapes/LineABShape';
import { MidpointShape } from './shapes/MidpointShape';

export enum ObjectType {
    FixedPoint = 'FixedPoint',
    FreePoint = 'FreePoint',
    Midpoint = 'Midpoint',
    IntersectionPoint = 'IntersectionPoint',
    SlidingPoint = 'SlidingPoint',
    LineAB = 'LineAB',
    Parameter = 'Parameter',
    Invariant = 'Invariant',
    Locus = 'Locus',
}

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
    LineAB = 'LineAB',
    Parameter = 'Parameter',
    Invariant = 'Invariant',
    Locus = 'Locus',
}

export enum ArgumentType {
    GridPoint = 'GridPoint',
    MobilePoint = 'MobilePoint',
    AnyDefinedPoint = 'AnyDefinedPoint',
    IntersectionPoint = 'IntersectionPoint',
    SlidingPoint = 'SlidingPoint',
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
            return currentActionStep === 0 ? { point1: value } : { point2: value };
        case ObjectType.Locus:
            return { point: value };
        default:
            return { value: value };
    }
}

export function getOccupiedPoints(dbObject: PartialDBObject): Vector2d[] {
    switch (dbObject.object_type) {
        case ObjectType.Midpoint:
            return new MidpointShape(dbObject, []).points;
        case ObjectType.LineAB:
            return new LineABShape(dbObject, []).points;
        default:
            return [];
    }
}