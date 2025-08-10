import type { Shape, TwoLineAngleInvariantProperties, ShapeCreatorInput, ArgumentValue, ObjectProperties, Line } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { PointBasedShape } from './PointBasedShape';
import { CanvasTwoLineAngleInvariant } from './CanvasComponents';
import { LineBasedShape } from './LineBasedShape';
import { BaseShapeCreator } from './BaseShape';
import { getShapeNameOrPoint, getPointsFromInput, intersectLines } from '../utils';
import { InitialPointShape } from './InitialPointShape';

export class TwoLineAngleInvariantShape extends PointBasedShape {
    objectType: ObjectType = ObjectType.TwoLineAngleInvariant;
    point: Vector2d;
    line1: Line;
    line2: Line;

    constructor(name: string, description: string, line1: Line, line2: Line) {
        super(name, description);
        this.line1 = line1;
        this.line2 = line2;

        // Calculate intersection point
        const intersectionPoint = intersectLines(line1, line2);
        if (intersectionPoint === null) {
            throw new Error(`Lines are parallel and do not intersect`);
        }
        this.point = intersectionPoint;
    }

    getActionType(): ActionType | null {
        return ActionType.AngleInvariant;
    }

    getDefinedPoint(): Vector2d | null {
        return this.point;
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasTwoLineAngleInvariant key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new TwoLineAngleInvariantShape(this.name, this.description, this.line1, this.line2);
    }
}

export class TwoLineAngleInvariantShapeCreator extends BaseShapeCreator {
    objectType = ObjectType.TwoLineAngleInvariant;

    getDBObjectProperties(input: ShapeCreatorInput): TwoLineAngleInvariantProperties {
        return {
            line1: getShapeNameOrPoint(input.argumentValues[0]?.[0]),
            line2: getShapeNameOrPoint(input.argumentValues[1]?.[0]),
        };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const twoLineAngleInvariantProperties = properties as TwoLineAngleInvariantProperties;
        const line1 = shapes.find(shape => shape.name === twoLineAngleInvariantProperties.line1) as LineBasedShape;
        const line2 = shapes.find(shape => shape.name === twoLineAngleInvariantProperties.line2) as LineBasedShape;

        if (line1 && line2) {
            return [[line1], [line2]];
        }
        return [];
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        const points = getPointsFromInput(input);

        // Check if both argument values are LineBasedShape instances
        const line1 = input.argumentValues[0]?.[0];
        const line2 = input.argumentValues[1]?.[0];

        if (line1 instanceof LineBasedShape && line2 instanceof LineBasedShape) {
            return new TwoLineAngleInvariantShape(
                input.objectName,
                this.getDescription(input),
                line1.getDefinedLine()!,
                line2.getDefinedLine()!
            );
        } else if (points.length === 1) {
            return new InitialPointShape(input.objectName, points[0]);
        } else {
            return null;
        }
    }

    protected getDescriptionInner(_input: ShapeCreatorInput, argumentStringValues: string[]): string {
        const line1Name = argumentStringValues[0] ?? '?';
        const line2Name = argumentStringValues[1] ?? '?';
        return `α(${line1Name}, ${line2Name}) = const`;
    }
} 