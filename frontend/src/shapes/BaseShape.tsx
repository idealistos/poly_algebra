import type { Shape, PartialDBObject, CanvasProperties, ShapeCreator, ShapeCreatorInput, ObjectProperties, ArgumentValue, DBObject } from '../types';
import { ShapeState, ActionType, ObjectType } from '../enums';
import React from 'react';
import type { Vector2d } from 'konva/lib/types';
import { getActionIcon } from '../actionIcons';

export abstract class BaseShape implements Shape {
    abstract objectType: ObjectType | null;
    name: string;
    description: string;
    state: ShapeState;

    constructor(name: string, description: string) {
        this.name = name;
        this.description = description;
        this.state = ShapeState.Default;
    }

    abstract getActionType(): ActionType | null;
    abstract getCanvasShape(canvasProperties?: CanvasProperties, key?: string): React.ReactNode;
    abstract getCoveredPoints(): { x: number; y: number }[];

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    matchesLastArgumentOf(_objectType: ObjectType): boolean {
        // This method is only needed for disambiguation of the last argument of an action
        return false;
    }

    getIcon(): React.ReactNode | null {
        const actionType = this.getActionType();
        if (!actionType) return null;
        return getActionIcon(actionType);
    }

    getDefinedPoint(): Vector2d | null {
        return null;
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    distanceToPoint(_point: Vector2d): number {
        return Infinity;
    }

    closeToPoint(point: Vector2d, delta: number): boolean {
        return this.distanceToPoint(point) < delta;
    }

    clone(): Shape {
        const shape = this.createClone();
        shape.state = this.state;
        return shape;
    }

    protected abstract createClone(): Shape;
}

export abstract class BaseShapeCreator implements ShapeCreator {
    abstract objectType: ObjectType;
    abstract getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties;
    abstract createShape(input: ShapeCreatorInput): Shape | null;

    getDBObject(input: ShapeCreatorInput): PartialDBObject {
        return {
            name: input.objectName,
            object_type: this.objectType,
            properties: this.getDBObjectProperties(input),
        };
    }

    getInputForDBObject(dbObject: DBObject, shapes: Shape[]): ShapeCreatorInput {
        let locusOrdinal = null;
        if (dbObject.object_type === ObjectType.Locus) {
            const existingLocusShapes = shapes.filter(s => s.objectType === ObjectType.Locus);
            const matchingShape = existingLocusShapes.find(s => s.name === dbObject.name);

            if (matchingShape) {
                // If this shape already exists, use its index
                locusOrdinal = existingLocusShapes.indexOf(matchingShape);
            } else {
                // If this is a new shape, use the count of existing locus shapes
                locusOrdinal = existingLocusShapes.length;
            }
        }
        return {
            objectName: dbObject.name,
            validatedExpressions: [],
            expressionValues: [],
            argumentValues: this.getArgumentValues(dbObject.properties, shapes),
            hintedObjectPoint: null,
            locusOrdinal,
        };
    }

    getDescription(input: ShapeCreatorInput): string {
        if (input.hintedObjectPoint != null) {
            return "";
        }
        const argumentStringValues = input.argumentValues.map(arg => {
            if (arg[0] == null) {
                return "";
            } else if (arg[0] instanceof BaseShape) {
                return arg[0].name;
            } else {
                const point = arg[0] as Vector2d;
                return `(${point.x}, ${point.y})`;
            }
        });
        return this.getDescriptionInner(input, argumentStringValues);
    }

    protected abstract getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[];
    protected abstract getDescriptionInner(input: ShapeCreatorInput, argumentStringValues: string[]): string;
}
