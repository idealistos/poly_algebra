import type { Shape, ComputedPointProperties, ObjectProperties, ShapeCreatorInput, ArgumentValue, DBObject } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShapeCreator } from './BaseShape';
import { CanvasComputedPoint } from './CanvasComponents';
import { PointBasedShape } from './PointBasedShape';

export class ComputedPointShape extends PointBasedShape {
    objectType: ObjectType = ObjectType.ComputedPoint;
    point: Vector2d;

    constructor(name: string, description: string, point: Vector2d) {
        super(name, description);
        this.point = point;
    }

    getActionType(): ActionType | null {
        return ActionType.ComputedPoint;
    }

    getDefinedPoint(): Vector2d | null {
        return this.point;
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasComputedPoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new ComputedPointShape(this.name, this.description, this.point);
    }
}

export class ComputedPointShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.ComputedPoint;

    getDBObjectProperties(input: ShapeCreatorInput): ComputedPointProperties {
        return {
            x_expr: input.validatedExpressions[0],
            y_expr: input.validatedExpressions[1],
            value: `${input.expressionValues[0]},${input.expressionValues[1]}`
        };
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    getInputForDBObject(dbObject: DBObject, _shapes: Shape[]): ShapeCreatorInput {
        const properties = dbObject.properties as ComputedPointProperties;
        const match = properties.value.match(/^(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)\s*,\s*(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)$/);
        const point = match ? { x: parseFloat(match[1]), y: parseFloat(match[2]) } : null;
        if (point == null) {
            throw new Error(`Invalid point value: ${properties.value}`);
        }
        return {
            objectName: dbObject.name,
            validatedExpressions: [properties.x_expr, properties.y_expr],
            expressionValues: [point.x, point.y],
            argumentValues: [],
            hintedObjectPoint: null,
            locusOrdinal: null
        }
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    getArgumentValues(_properties: ObjectProperties): ArgumentValue[] {
        throw new Error('getArgumentValues() should not be called for ComputedPointShape');
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        const point = { x: input.expressionValues[0], y: input.expressionValues[1] };
        return new ComputedPointShape(input.objectName, this.getDescription(input), point);
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    protected getDescriptionInner(input: ShapeCreatorInput, _argumentStringValues: string[]): string {
        return `${input.objectName} (${input.validatedExpressions[0]}, ${input.validatedExpressions[1]})`;
    }
} 