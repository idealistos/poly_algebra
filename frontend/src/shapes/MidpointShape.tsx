import type { Shape, PartialDBObject, MidpointProperties } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasMidpoint } from './CanvasComponents';
import { getPointDescription, parsePoint } from '../utils';

export class MidpointShape extends BaseShape {
    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];

        const properties = dbObject.properties as Partial<MidpointProperties>;
        const point1 = properties.point1;
        const point2 = properties.point2;

        // Parse point1
        if (point1) {
            const point1Coords = parsePoint(point1, shapes);
            if (point1Coords) {
                this.points.push(point1Coords);
            }
        }

        // Parse point2
        if (point2) {
            const point2Coords = parsePoint(point2, shapes);
            if (point2Coords) {
                this.points.push(point2Coords);
            }
        }
    }

    getActionType(): ActionType | null {
        return ActionType.Midpoint;
    }

    getDescription(): string {
        const properties = this.dbObject.properties as Partial<MidpointProperties>;
        const point1 = getPointDescription(properties.point1 ?? null);
        const point2 = getPointDescription(properties.point2 ?? null);
        return `${this.dbObject.name}: midpoint(${point1}, ${point2})`;
    }

    getDefinedPoint(): Vector2d | null {
        if (this.points.length < 2) return null;
        return {
            x: (this.points[0].x + this.points[1].x) / 2,
            y: (this.points[0].y + this.points[1].y) / 2
        };
    }

    distanceToPoint(point: Vector2d): number {
        const definedPoint = this.getDefinedPoint();
        if (!definedPoint) return Infinity;
        return Math.sqrt(
            Math.pow(point.x - definedPoint.x, 2) + Math.pow(point.y - definedPoint.y, 2)
        );
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasMidpoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new MidpointShape(this.dbObject, []);
    }
} 