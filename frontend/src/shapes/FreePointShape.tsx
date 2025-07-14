import type { Shape, PartialDBObject } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasFreePoint } from './CanvasComponents';

export class FreePointShape extends BaseShape {
    constructor(dbObject: PartialDBObject) {
        super(dbObject);
        const value = (dbObject.properties as { value?: string })?.value;
        const match = value?.match(/^(-?\d+),\s*(-?\d+)$/);
        this.points = match ? [{ x: parseInt(match[1]), y: parseInt(match[2]) }] : [];
    }

    getActionType(): ActionType | null {
        return ActionType.FreePoint;
    }

    getDescription(): string {
        return `${this.dbObject.name} (${this.points[0]?.x ?? 0}, ${this.points[0]?.y ?? 0})`;
    }

    getDefinedPoint(): Vector2d | null {
        return this.points[0] || null;
    }

    distanceToPoint(point: Vector2d): number {
        if (this.points.length === 0) return Infinity;
        return Math.sqrt(
            Math.pow(point.x - this.points[0].x, 2) + Math.pow(point.y - this.points[0].y, 2)
        );
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasFreePoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new FreePointShape(this.dbObject);
    }
} 