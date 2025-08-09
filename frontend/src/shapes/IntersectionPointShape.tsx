import type { Shape, IntersectionPointProperties, ShapeCreatorInput, ArgumentValue, ObjectProperties } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { PointBasedShape } from './PointBasedShape';
import { CanvasIntersectionPoint } from './CanvasComponents';
import { LineBasedShape } from './LineBasedShape';
import { BaseShapeCreator } from './BaseShape';
import { getShapeNameOrPoint, getPointsFromInput } from '../utils';
import { InitialPointShape } from './InitialPointShape';

export class IntersectionPointShape extends PointBasedShape {
    objectType: ObjectType = ObjectType.IntersectionPoint;
    point: Vector2d;


    constructor(name: string, description: string, point: Vector2d) {
        super(name, description);
        this.point = point;
    }

    getActionType(): ActionType | null {
        return ActionType.IntersectionPoint;
    }

    getDefinedPoint(): Vector2d | null {
        return this.point;
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasIntersectionPoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new IntersectionPointShape(this.name, this.description, this.point);
    }
}

export class IntersectionPointShapeCreator extends BaseShapeCreator {
    objectType = ObjectType.IntersectionPoint;

    getDBObjectProperties(input: ShapeCreatorInput): IntersectionPointProperties {
        return {
            object_name_1: getShapeNameOrPoint(input.argumentValues[0]?.[0]),
            object_name_2: getShapeNameOrPoint(input.argumentValues[0]?.[1]),
        };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const intersectionPointProperties = properties as IntersectionPointProperties;
        const line1 = shapes.find(shape => shape.name === intersectionPointProperties.object_name_1) as LineBasedShape;
        const line2 = shapes.find(shape => shape.name === intersectionPointProperties.object_name_2) as LineBasedShape;
        return [[line1, line2]];
    }


    createShape(input: ShapeCreatorInput): Shape | null {
        const points = getPointsFromInput(input);
        if (points.length === 0 && input.argumentValues.length === 0) {
            return null;
        } else if (points.length === 0 && input.argumentValues[0]?.[0] instanceof LineBasedShape && input.argumentValues[0]?.[1] instanceof LineBasedShape) {
            const line1 = input.argumentValues[0]?.[0] as LineBasedShape;
            const line2 = input.argumentValues[0]?.[1] as LineBasedShape;
            const point = line1.intersect(line2);
            if (point == null) {
                throw new Error('Lines are parallel');
            }
            return new IntersectionPointShape(input.objectName, this.getDescription(input), point);
        } else if (points.length == 1) {
            return new InitialPointShape(input.objectName, points[0]);
        } else {
            throw new Error('Invalid input');
        }


    }

    protected getDescriptionInner(input: ShapeCreatorInput, argumentStringValues: string[]): string {
        const line1Name = argumentStringValues[0] ?? '?';
        const line2Name = (input.argumentValues[0]?.[1] as Shape).name;
        return `${input.objectName} (${line1Name} × ${line2Name})`;
    }
} 