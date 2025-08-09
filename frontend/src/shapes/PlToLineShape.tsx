import { LineBasedShape } from './LineBasedShape';
import type { Shape, PlToLineProperties, Line, CanvasProperties, ObjectProperties, ShapeCreatorInput, ArgumentValue } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import type { Vector2d } from 'konva/lib/types';
import { getShapeNameOrPoint, getDefinedOrGridPoint, getPointsFromInput } from '../utils';
import { CanvasPlToLine } from './CanvasComponents';
import { BaseShapeCreator } from './BaseShape';
import { InitialPointShape } from './InitialPointShape';
import { LineBasedShape as LineBasedShapeClass } from './LineBasedShape';
import { ProjectionShape } from './ProjectionShape';

export class PlToLineShape extends LineBasedShape {
    objectType: ObjectType = ObjectType.PlToLine;
    point: Vector2d;
    referenceLine: Line;

    constructor(name: string, description: string, point: Vector2d, referenceLine: Line) {
        super(name, description);
        this.point = point;
        this.referenceLine = referenceLine;
    }

    getActionType(): ActionType | null {
        return ActionType.PlToLine;
    }

    getDefinedLine(): Line | null {
        return {
            point: this.point,
            n: this.referenceLine.n
        };
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasPlToLine key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new PlToLineShape(this.name, this.description, this.point, this.referenceLine);
    }
}

export class PlToLineShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.PlToLine;

    getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties {
        return {
            point: getShapeNameOrPoint(input.argumentValues[0]?.[0]),
            line: (input.argumentValues[1]?.[0] as Shape).name,
        };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const plToLineProperties = properties as PlToLineProperties;
        const point = getDefinedOrGridPoint(plToLineProperties.point, shapes);
        const line = shapes.find(shape => shape.name === plToLineProperties.line);
        if (point == null || line == null) {
            throw new Error('Invalid point or line value');
        }
        return [[point], [line]];
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        const points = getPointsFromInput(input);
        const lineShape = input.argumentValues[1]?.[0] as LineBasedShapeClass;
        const line = lineShape?.getDefinedLine() || null;

        if (points.length == 0) {
            return null;
        } else if (points.length == 1) {
            if (line == null) {
                return new InitialPointShape(input.objectName, points[0]);
            }
            return new PlToLineShape(input.objectName, this.getDescription(input), points[0], line);
        } else {
            // Hinted
            return new ProjectionShape(input.objectName, "", points[0], points[1], null);
        }
    }

    protected getDescriptionInner(input: ShapeCreatorInput, argumentStringValues: string[]): string {
        const lineName = (input.argumentValues[1]?.[0] as Shape)?.name || 'unknown';
        return `${input.objectName} ∥ ${lineName} through ${argumentStringValues[0]}`;
    }
} 