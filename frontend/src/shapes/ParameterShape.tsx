import type { ArgumentValue, ObjectProperties, Shape, ShapeCreatorInput } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import { BaseShape, BaseShapeCreator } from './BaseShape';

export class ParameterShape extends BaseShape {
    objectType: ObjectType = ObjectType.Parameter;

    constructor(name: string, description: string) {
        super(name, description);
    }

    getActionType(): ActionType | null {
        return ActionType.Parameter;
    }

    getCoveredPoints(): { x: number; y: number }[] {
        return [];
    }

    getCanvasShape(): React.ReactNode {
        // Parameters don't have visual representation on canvas
        return null;
    }

    protected createClone(): Shape {
        return new ParameterShape(this.name, this.description);
    }
}

export class ParameterShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.Parameter;

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    getDBObjectProperties(_input: ShapeCreatorInput): ObjectProperties {
        return {};
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    getArgumentValues(_properties: ObjectProperties, _shapes: Shape[]): ArgumentValue[] {
        return [];
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        return new ParameterShape(input.objectName, this.getDescription(input));
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    protected getDescriptionInner(input: ShapeCreatorInput, _argumentStringValues: string[]): string {
        return input.objectName;
    }
}