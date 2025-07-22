import type { Shape, PartialDBObject, PpBisectorProperties, Line } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { LineBasedShape } from './LineBasedShape';
import { CanvasPpBisector } from './CanvasComponents';
import { getPointDescription, parsePoint } from '../utils';

export class PpBisectorShape extends LineBasedShape {
    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];
        const properties = dbObject.properties as Partial<PpBisectorProperties>;
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
        return ActionType.PpBisector;
    }

    getDescription(): string {
        const properties = this.dbObject.properties as Partial<PpBisectorProperties>;
        const point1 = getPointDescription(properties.point1 ?? null);
        const point2 = getPointDescription(properties.point2 ?? null);
        return `${this.dbObject.name} bisecting (${point1}, ${point2})`;
    }

    getDefinedLine(): Line | null {
        if (this.points.length < 2) return null;

        const p1 = this.points[0];
        const p2 = this.points[1];

        // Calculate midpoint
        const midX = (p1.x + p2.x) / 2;
        const midY = (p1.y + p2.y) / 2;
        const midpoint = { x: midX, y: midY };

        // Calculate direction vector of the original line segment
        const dx = p2.x - p1.x;
        const dy = p2.y - p1.y;

        // Calculate perpendicular vector n (perpendicular bisector)
        const n = { x: dx, y: dy };

        return {
            point: midpoint,
            n,
        };
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasPpBisector key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new PpBisectorShape(this.dbObject, []);
    }
} 