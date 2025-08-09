import type { Shape, ObjectProperties, ShapeCreatorInput, FreePointProperties, ArgumentValue } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShapeCreator } from './BaseShape';
import { CanvasFreePoint } from './CanvasComponents';
import { getGridOrHintedPointFromInput, getPointsFromInput, parsePoint } from '../utils';
import { PointBasedShape } from './PointBasedShape';

export class FreePointShape extends PointBasedShape {
    objectType: ObjectType = ObjectType.FreePoint;
    point: Vector2d;

    constructor(name: string, description: string, point: Vector2d) {
        super(name, description);
        this.point = point;
    }

    getActionType(): ActionType | null {
        return ActionType.FreePoint;
    }

    getDefinedPoint(): Vector2d | null {
        return this.point;
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasFreePoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new FreePointShape(this.name, this.description, this.point);
    }
}

export class FreePointShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.FreePoint;

    getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties {
        const point = getGridOrHintedPointFromInput(input);
        if (point == null) {
            throw new Error(`Invalid input: ${JSON.stringify(input)}`);
        }
        return { value: `${point.x},${point.y}` };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const value = (properties as FreePointProperties).value;
        const parsedPoint = parsePoint(value, shapes);
        if (parsedPoint == null) {
            throw new Error('Invalid point value');
        }
        return [[parsedPoint]];
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        const points = getPointsFromInput(input);
        if (points.length == 0) {
            return null;
        }
        return new FreePointShape(input.objectName, this.getDescription(input), points[0]);
    }

    protected getDescriptionInner(input: ShapeCreatorInput, argumentStringValues: string[]): string {
        return `${input.objectName} ${argumentStringValues[0]}`;
    }
} 