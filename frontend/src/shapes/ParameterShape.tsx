import type { Shape, PartialDBObject } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import { BaseShape } from './BaseShape';

export class ParameterShape extends BaseShape {
    constructor(dbObject: PartialDBObject) {
        super(dbObject);
        this.points = [];
    }

    getActionType(): ActionType | null {
        return ActionType.Parameter;
    }

    getDescription(): string {
        return this.dbObject.name;
    }

    getCanvasShape(): React.ReactNode {
        // Parameters don't have visual representation on canvas
        return null;
    }

    protected createClone(): Shape {
        return new ParameterShape(this.dbObject);
    }
} 