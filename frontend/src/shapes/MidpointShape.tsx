import type { Shape, MidpointProperties, ObjectProperties, ShapeCreatorInput, ArgumentValue } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShapeCreator } from './BaseShape';
import { CanvasMidpoint } from './CanvasComponents';
import { getDefinedOrGridPoint, getPointsFromInput, getShapeNameOrPoint } from '../utils';
import { InitialPointShape } from './InitialPointShape';
import { PointBasedShape } from './PointBasedShape';

export class MidpointShape extends PointBasedShape {
    objectType: ObjectType = ObjectType.Midpoint;
    point1: Vector2d;
    point2: Vector2d;

    constructor(name: string, description: string, point1: Vector2d, point2: Vector2d) {
        super(name, description);
        this.point1 = point1;
        this.point2 = point2;
    }

    getActionType(): ActionType | null {
        return ActionType.Midpoint;
    }

    getDefinedPoint(): Vector2d | null {
        return {
            x: (this.point1.x + this.point2.x) / 2,
            y: (this.point1.y + this.point2.y) / 2
        };
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasMidpoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new MidpointShape(this.name, this.description, this.point1, this.point2);
    }
}

export class MidpointShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.Midpoint;

    getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties {
        return {
            point1: getShapeNameOrPoint(input.argumentValues[0]?.[0]),
            point2: getShapeNameOrPoint(input.argumentValues[1]?.[0]),
        };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const midpointProperties = properties as MidpointProperties;
        const point1 = getDefinedOrGridPoint(midpointProperties.point1, shapes);
        const point2 = getDefinedOrGridPoint(midpointProperties.point2, shapes);
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
            return new MidpointShape(input.objectName, this.getDescription(input), points[0], points[1]);
        }
    }

    protected getDescriptionInner(input: ShapeCreatorInput, argumentStringValues: string[]): string {
        return `${input.objectName}: midpoint(${argumentStringValues[0]}, ${argumentStringValues[1]})`;
    }
} 