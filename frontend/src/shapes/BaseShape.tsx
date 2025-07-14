import type { Shape, PartialDBObject, CanvasProperties } from '../types';
import { ShapeState, ActionType } from '../enums';
import React from 'react';
import type { Vector2d } from 'konva/lib/types';
import { getActionIcon } from '../actionIcons';

export abstract class BaseShape implements Shape {
    dbObject: PartialDBObject;
    state: ShapeState;
    points: { x: number; y: number }[];

    constructor(dbObject: PartialDBObject) {
        this.dbObject = dbObject;
        this.state = ShapeState.Default;
        this.points = [];
    }

    abstract getActionType(): ActionType | null;
    abstract getDescription(): string;
    abstract getCanvasShape(canvasProperties?: CanvasProperties, key?: string): React.ReactNode;

    getCoveredPoints(): { x: number; y: number }[] {
        return this.points;
    }

    getIcon(): React.ReactNode | null {
        const actionType = this.getActionType();
        if (!actionType) return null;
        return getActionIcon(actionType);
    }

    getDBObjectForNextStep(): PartialDBObject | null {
        return null;
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
        shape.points = [...this.points];
        return shape;
    }

    protected abstract createClone(): Shape;
} 