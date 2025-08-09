import type { Shape, SlidingPointProperties, Line, ObjectProperties, ShapeCreatorInput, ArgumentValue } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { CanvasSlidingPoint } from './CanvasComponents';
import { LineBasedShape } from './LineBasedShape';
import { getShapeNameOrPoint, getDefinedOrGridPoint, getPointsFromInput } from '../utils';
import { BaseShapeCreator } from './BaseShape';
import { InitialPointShape } from './InitialPointShape';
import { PointBasedShape } from './PointBasedShape';

export class SlidingPointShape extends PointBasedShape {
    objectType: ObjectType = ObjectType.SlidingPoint;
    gridPoint: Vector2d;
    line: Line;

    constructor(name: string, description: string, gridPoint: Vector2d, line: Line) {
        super(name, description);
        this.gridPoint = gridPoint;
        this.line = line;
    }

    getActionType(): ActionType | null {
        return ActionType.SlidingPoint;
    }

    getDefinedPoint(): Vector2d | null {
        return this.gridPoint;
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasSlidingPoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new SlidingPointShape(this.name, this.description, this.gridPoint, this.line);
    }
}

export class SlidingPointShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.SlidingPoint;

    getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties {
        return {
            value: getShapeNameOrPoint(input.argumentValues[0]?.[0]),
            constraining_object_name: (input.argumentValues[0]?.[1] as Shape).name,
        };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const slidingPointProperties = properties as SlidingPointProperties;
        const gridPoint = getDefinedOrGridPoint(slidingPointProperties.value, shapes);
        const line = shapes.find(shape => shape.name === slidingPointProperties.constraining_object_name);
        if (gridPoint == null || line == null || !(line instanceof LineBasedShape)) {
            throw new Error('Invalid gridPoint or line value');
        }
        return [[gridPoint, line]];
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        const points = getPointsFromInput(input);
        const line = (input.argumentValues[0]?.[1] instanceof LineBasedShape) ?
            (input.argumentValues[0]?.[1] as LineBasedShape).getDefinedLine() : null;
        if (points.length == 0) {
            return null;
        } else {
            if (line == null) {
                return new InitialPointShape(input.objectName, points[0]);
            }
            return new SlidingPointShape(input.objectName, this.getDescription(input), points[0], line);
        }
    }

    protected getDescriptionInner(input: ShapeCreatorInput, argumentStringValues: string[]): string {
        const lineName = (input.argumentValues[0]?.[1] as Shape)?.name || 'undefined';
        return `${input.objectName} (${argumentStringValues[0]}) on ${lineName}`;
    }
} 