import type { Shape, PartialDBObject, TwoPointDistanceInvariantProperties } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasTwoPointDistanceInvariant } from './CanvasComponents';
import { getPointDescription, distanceToLineSegment, parsePoint } from '../utils';

export class TwoPointDistanceInvariantShape extends BaseShape {
    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];

        const properties = dbObject.properties as Partial<TwoPointDistanceInvariantProperties>;
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
        return ActionType.DistanceInvariant;
    }

    getDescription(): string {
        const properties = this.dbObject.properties as Partial<TwoPointDistanceInvariantProperties>;
        const point1 = getPointDescription(properties.point1 ?? null);
        const point2 = getPointDescription(properties.point2 ?? null);
        return `d(${point1}, ${point2}) = const`;
    }

    distanceToPoint(point: Vector2d): number {
        if (this.points.length < 2) return Infinity;

        const p1 = this.points[0];
        const p2 = this.points[1];

        return distanceToLineSegment(point, p1, p2);
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasTwoPointDistanceInvariant key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new TwoPointDistanceInvariantShape(this.dbObject, []);
    }
} 