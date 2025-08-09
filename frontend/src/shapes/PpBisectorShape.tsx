import type { Shape, PpBisectorProperties, Line, ObjectProperties, ShapeCreatorInput, ArgumentValue } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { LineBasedShape } from './LineBasedShape';
import { CanvasPpBisector } from './CanvasComponents';
import { getShapeNameOrPoint, getDefinedOrGridPoint, getPointsFromInput } from '../utils';
import { BaseShapeCreator } from './BaseShape';
import { InitialPointShape } from './InitialPointShape';

export class PpBisectorShape extends LineBasedShape {
    objectType: ObjectType = ObjectType.PpBisector;
    point1: Vector2d;
    point2: Vector2d;

    constructor(name: string, description: string, point1: Vector2d, point2: Vector2d) {
        super(name, description);
        this.point1 = point1;
        this.point2 = point2;
    }

    getActionType(): ActionType | null {
        return ActionType.PpBisector;
    }

    getDefinedLine(): Line | null {
        // Calculate midpoint
        const midX = (this.point1.x + this.point2.x) / 2;
        const midY = (this.point1.y + this.point2.y) / 2;
        const midpoint = { x: midX, y: midY };

        // Calculate direction vector of the original line segment
        const dx = this.point2.x - this.point1.x;
        const dy = this.point2.y - this.point1.y;

        // Calculate perpendicular vector n (perpendicular bisector)
        const n = { x: dx, y: dy };

        return {
            point: midpoint,
            n,
        };
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasPpBisector key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new PpBisectorShape(this.name, this.description, this.point1, this.point2);
    }
}

export class PpBisectorShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.PpBisector;

    getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties {
        return {
            point1: getShapeNameOrPoint(input.argumentValues[0]?.[0]),
            point2: getShapeNameOrPoint(input.argumentValues[1]?.[0]),
        };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const ppBisectorProperties = properties as PpBisectorProperties;
        const point1 = getDefinedOrGridPoint(ppBisectorProperties.point1, shapes);
        const point2 = getDefinedOrGridPoint(ppBisectorProperties.point2, shapes);
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
            return new PpBisectorShape(input.objectName, this.getDescription(input), points[0], points[1]);
        }
    }

    protected getDescriptionInner(input: ShapeCreatorInput, argumentStringValues: string[]): string {
        return `${input.objectName} bisecting (${argumentStringValues[0]}, ${argumentStringValues[1]})`;
    }
} 