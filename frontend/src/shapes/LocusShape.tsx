import type { Shape, LocusProperties, ObjectProperties, ShapeCreatorInput, ArgumentValue } from '../types';
import { ObjectType, ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { CanvasLocus } from './CanvasComponents';
import { BaseShape, BaseShapeCreator } from './BaseShape';
import { PointBasedShape } from './PointBasedShape';
import { getDefinedOrGridPoint, getPointsFromInput } from '../utils';

export class LocusShape extends PointBasedShape {
    objectType: ObjectType = ObjectType.Locus;
    point: Vector2d;
    locusOrdinal: number;

    constructor(name: string, description: string, point: Vector2d, locusOrdinal: number) {
        super(name, description);
        this.point = point;
        this.locusOrdinal = locusOrdinal;
    }

    getActionType(): ActionType | null {
        return ActionType.Locus;
    }

    getDefinedPoint(): Vector2d | null {
        return this.point;
    }

    distanceToPoint(point: Vector2d): number {
        return Math.sqrt(
            Math.pow(point.x - this.point.x, 2) + Math.pow(point.y - this.point.y, 2)
        );
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasLocus key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new LocusShape(this.name, this.description, this.point, this.locusOrdinal);
    }
}

export class LocusShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.Locus;

    getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties {
        const shape = input.argumentValues[0]?.[0];
        if (!(shape instanceof BaseShape) || shape.getDefinedPoint() == null) {
            throw new Error('Invalid point value');
        }
        return { point: shape.name };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const locusProperties = properties as LocusProperties;
        const point = getDefinedOrGridPoint(locusProperties.point, shapes);
        if (point == null) {
            throw new Error('Invalid point value');
        }
        return [[point]];
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        const points = getPointsFromInput(input);
        if (points.length == 0) {
            return null;
        }
        return new LocusShape(input.objectName, this.getDescription(input), points[0], input.locusOrdinal ?? 0);
    }

    protected getDescriptionInner(_input: ShapeCreatorInput, argumentStringValues: string[]): string {
        return `Plot ${argumentStringValues[0]}`;
    }
} 