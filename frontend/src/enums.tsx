import type { Shape } from './types';

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