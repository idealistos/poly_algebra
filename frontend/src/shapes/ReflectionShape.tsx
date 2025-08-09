import type { Shape, Line, CanvasProperties, ReflectionProperties, ObjectProperties, ShapeCreatorInput, ArgumentValue } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import { BaseShapeCreator } from './BaseShape';
import { getShapeNameOrPoint, getDefinedOrGridPoint, getPointsFromInput } from '../utils';
import type { Vector2d } from 'konva/lib/types';
import { CanvasReflection } from './CanvasComponents';
import { LineBasedShape } from './LineBasedShape';
import { InitialPointShape } from './InitialPointShape';
import { PointBasedShape } from './PointBasedShape';

export class ReflectionShape extends PointBasedShape {
    objectType: ObjectType = ObjectType.Reflection;
    point1: Vector2d;
    point2: Vector2d;
    line: Line | null;

    constructor(name: string, description: string, point1: Vector2d, point2: Vector2d | null, line: Line | null) {
        super(name, description);
        this.point1 = point1;
        if (point2 == null) {
            if (line == null) {
                throw new Error("Both point2 and line are null");
            }
            this.point2 = this.findReflectedPoint(point1, line);
        } else {
            this.point2 = point2;
        }
        this.line = line;
    }

    getActionType(): ActionType {
        return ActionType.Reflection;
    }

    getDefinedPoint(): Vector2d | null {
        return this.point2;
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasReflection key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new ReflectionShape(this.name, this.description, this.point1, this.point2, this.line);
    }

    private findReflectedPoint(point: Vector2d, line: Line): Vector2d {
        // Vector from line point to the point to reflect
        const dx = point.x - line.point.x;
        const dy = point.y - line.point.y;

        // Dot product: n · (point - line.point)
        const dotProduct = line.n.x * dx + line.n.y * dy;

        // Magnitude squared of normal vector: |n|²
        const nMagnitudeSquared = line.n.x * line.n.x + line.n.y * line.n.y;

        if (nMagnitudeSquared === 0) {
            // Line has zero normal vector, return the original point
            return { x: point.x, y: point.y };
        }

        // Calculate the reflection
        // reflection = point - 2 * (dotProduct / |n|²) × n
        const scale = dotProduct / nMagnitudeSquared;
        const reflectedX = point.x - 2 * scale * line.n.x;
        const reflectedY = point.y - 2 * scale * line.n.y;

        return { x: reflectedX, y: reflectedY };
    }
}

export class ReflectionShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.Reflection;

    getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties {
        return {
            point: getShapeNameOrPoint(input.argumentValues[0]?.[0]),
            line: (input.argumentValues[1]?.[0] as Shape).name,
        };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const reflectionProperties = properties as ReflectionProperties;
        const point = getDefinedOrGridPoint(reflectionProperties.point, shapes);
        const line = shapes.find(shape => shape.name === reflectionProperties.line);
        if (point == null || line == null) {
            throw new Error('Invalid point or line value');
        }
        return [[point], [line]];
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        const points = getPointsFromInput(input);
        const line = (input.argumentValues[1]?.[0] instanceof LineBasedShape) ?
            (input.argumentValues[1]?.[0] as LineBasedShape).getDefinedLine() : null;
        if (points.length == 0) {
            return null;
        } else if (points.length == 1) {
            if (line == null) {
                return new InitialPointShape(input.objectName, points[0]);
            }
            return new ReflectionShape(input.objectName, this.getDescription(input), points[0], null, line);
        } else {
            return new ReflectionShape(input.objectName, this.getDescription(input), points[0], points[1], null);
        }
    }

    protected getDescriptionInner(input: ShapeCreatorInput, argumentStringValues: string[]): string {
        return `${input.objectName}: reflection(${argumentStringValues[0]}, ${argumentStringValues[1]})`;
    }
} 