import type { Shape, PartialDBObject, InvariantProperties } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import { BaseShape } from './BaseShape';

export class InvariantShape extends BaseShape {
    constructor(dbObject: PartialDBObject) {
        super(dbObject);
        this.points = [];
    }

    getActionType(): ActionType | null {
        return ActionType.Invariant;
    }

    getDescription(): string {
        const formula = (this.dbObject.properties as Partial<InvariantProperties>)?.formula ?? '?';
        return `${formula} = const`;
    }

    getCanvasShape(): React.ReactNode {
        // Invariants don't have visual representation on canvas
        return null;
    }

    protected createClone(): Shape {
        return new InvariantShape(this.dbObject);
    }
} 