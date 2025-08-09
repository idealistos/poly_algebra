import type { Shape, ShapeCreatorInput, ObjectProperties, ArgumentValue, DBObject, InvariantProperties } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import { BaseShape, BaseShapeCreator } from './BaseShape';

export class InvariantShape extends BaseShape {
    objectType: ObjectType = ObjectType.Invariant;

    getActionType(): ActionType | null {
        return ActionType.Invariant;
    }

    getCanvasShape(): React.ReactNode {
        // Invariants don't have visual representation on canvas
        return null;
    }

    getCoveredPoints(): { x: number; y: number }[] {
        return [];
    }

    protected createClone(): Shape {
        return new InvariantShape(this.name, this.description);
    }
}

export class InvariantShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.Invariant;

    getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties {
        const expression = input.validatedExpressions[0];
        if (expression == null) {
            throw new Error(`Invalid input: ${JSON.stringify(input)}`);
        }
        return { formula: expression };
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    getInputForDBObject(dbObject: DBObject, _shapes: Shape[]): ShapeCreatorInput {
        const formula = (dbObject.properties as InvariantProperties).formula;
        return {
            objectName: dbObject.name,
            validatedExpressions: [formula],
            expressionValues: [],
            argumentValues: [],
            hintedObjectPoint: null,
            locusOrdinal: null,
        }
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    getArgumentValues(_properties: ObjectProperties, _shapes: Shape[]): ArgumentValue[] {
        throw new Error("getArgumentValues() is not needed for InvariantShapeCreator because getInputForDBObject() is overridden");
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        return new InvariantShape(input.objectName, this.getDescription(input));
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    protected getDescriptionInner(input: ShapeCreatorInput, _argumentStringValues: string[]): string {
        return `${input.validatedExpressions[0]} = const`;
    }
}