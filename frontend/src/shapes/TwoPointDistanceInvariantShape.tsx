import type { Shape, TwoPointDistanceInvariantProperties, ObjectProperties, ShapeCreatorInput, ArgumentValue } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape, BaseShapeCreator } from './BaseShape';
import { CanvasTwoPointDistanceInvariant } from './CanvasComponents';
import { distanceToLineSegment, getShapeNameOrPoint, getDefinedOrGridPoint, getPointsFromInput } from '../utils';
import { InitialPointShape } from './InitialPointShape';

export class TwoPointDistanceInvariantShape extends BaseShape {
    objectType: ObjectType = ObjectType.TwoPointDistanceInvariant;
    point1: Vector2d;
    point2: Vector2d;

    constructor(name: string, description: string, point1: Vector2d, point2: Vector2d) {
        super(name, description);
        this.point1 = point1;
        this.point2 = point2;
    }

    getActionType(): ActionType | null {
        return ActionType.DistanceInvariant;
    }

    getCoveredPoints(): { x: number; y: number }[] {
        return [];
    }

    distanceToPoint(point: Vector2d): number {
        return distanceToLineSegment(point, this.point1, this.point2);
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasTwoPointDistanceInvariant key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new TwoPointDistanceInvariantShape(this.name, this.description, this.point1, this.point2);
    }
}

export class TwoPointDistanceInvariantShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.TwoPointDistanceInvariant;

    getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties {
        return {
            point1: getShapeNameOrPoint(input.argumentValues[0]?.[0]),
            point2: getShapeNameOrPoint(input.argumentValues[1]?.[0]),
        };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const twoPointDistanceProperties = properties as TwoPointDistanceInvariantProperties;
        const point1 = getDefinedOrGridPoint(twoPointDistanceProperties.point1, shapes);
        const point2 = getDefinedOrGridPoint(twoPointDistanceProperties.point2, shapes);
        if (point1 == null || point2 == null) {
            throw new Error('Invalid point1 or point2 value');
        }
        return [[point1], [point2]];
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        const points = getPointsFromInput(input);
        if (points.length == 0) {
            return null;
        } else if (points.length == 1) {
            return new InitialPointShape(input.objectName, points[0]);
        } else {
            return new TwoPointDistanceInvariantShape(input.objectName, this.getDescription(input), points[0], points[1]);
        }
    }

    protected getDescriptionInner(_input: ShapeCreatorInput, argumentStringValues: string[]): string {
        return `d(${argumentStringValues[0]}, ${argumentStringValues[1]}) = const`;
    }
} 