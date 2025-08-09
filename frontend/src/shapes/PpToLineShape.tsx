import { LineBasedShape } from './LineBasedShape';
import type { Shape, PpToLineProperties, Line, CanvasProperties, ObjectProperties, ShapeCreatorInput, ArgumentValue } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import type { Vector2d } from 'konva/lib/types';
import { getShapeNameOrPoint, getDefinedOrGridPoint, getPointsFromInput } from '../utils';
import { CanvasPpToLine } from './CanvasComponents';
import { BaseShapeCreator } from './BaseShape';
import { InitialPointShape } from './InitialPointShape';
import { LineBasedShape as LineBasedShapeClass } from './LineBasedShape';
import { ProjectionShape } from './ProjectionShape';

export class PpToLineShape extends LineBasedShape {
    objectType: ObjectType = ObjectType.PpToLine;
    point: Vector2d;
    referenceLine: Line;


    constructor(name: string, description: string, point: Vector2d, referenceLine: Line) {
        super(name, description);
        this.referenceLine = referenceLine;
        this.point = this.findClosestPointOnLine(point, referenceLine);
    }

    getActionType(): ActionType | null {
        return ActionType.PpToLine;
    }

    getDefinedLine(): Line | null {
        return {
            point: this.point,
            n: { x: this.referenceLine.n.y, y: -this.referenceLine.n.x }
        };
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasPpToLine key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new PpToLineShape(this.name, this.description, this.point, this.referenceLine);
    }

    private findClosestPointOnLine(point: Vector2d, lineDef: Line): Vector2d {
        // Calculate the closest point on the line to the given point
        // This is the projection of the point onto the line
        const linePoint = lineDef.point;
        const lineNormal = lineDef.n;

        // Vector from line point to the given point
        const vectorToPoint = {
            x: point.x - linePoint.x,
            y: point.y - linePoint.y
        };

        // Project the vector onto the line normal
        const projection = (vectorToPoint.x * lineNormal.x + vectorToPoint.y * lineNormal.y) /
            (lineNormal.x * lineNormal.x + lineNormal.y * lineNormal.y);

        // The closest point is the line point plus the projection
        return {
            x: point.x - projection * lineNormal.x,
            y: point.y - projection * lineNormal.y
        };
    }
}

export class PpToLineShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.PpToLine;

    getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties {
        return {
            point: getShapeNameOrPoint(input.argumentValues[0]?.[0]),
            line: (input.argumentValues[1]?.[0] as Shape).name,
        };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const ppToLineProperties = properties as PpToLineProperties;
        const point = getDefinedOrGridPoint(ppToLineProperties.point, shapes);
        const line = shapes.find(shape => shape.name === ppToLineProperties.line);
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
            return new PpToLineShape(input.objectName, this.getDescription(input), points[0], line);
        } else {
            // Hinted
            return new ProjectionShape(input.objectName, "", points[0], points[1], null);
        }
    }

    protected getDescriptionInner(input: ShapeCreatorInput, argumentStringValues: string[]): string {
        const lineName = (input.argumentValues[1]?.[0] as Shape)?.name || 'unknown';
        return `${input.objectName} ⟂ ${lineName} through ${argumentStringValues[0]}`;
    }
} 