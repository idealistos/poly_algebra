import type { Shape, PartialDBObject, LineABProperties, Line } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { LineBasedShape } from './LineBasedShape';
import { CanvasLineAB } from './CanvasComponents';
import { getPointDescription, parsePoint } from '../utils';

export class LineABShape extends LineBasedShape {
    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];

        const properties = dbObject.properties as Partial<LineABProperties>;
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
        return ActionType.LineAB;
    }

    getDescription(): string {
        const properties = this.dbObject.properties as Partial<LineABProperties>;
        const point1 = getPointDescription(properties.point1 ?? null);
        const point2 = getPointDescription(properties.point2 ?? null);
        return `${this.dbObject.name} (${point1}, ${point2})`;
    }

    getDefinedLine(): Line | null {
        if (this.points.length < 2) return null;

        const p1 = this.points[0];
        const p2 = this.points[1];

        // Calculate direction vector of the line
        const dx = p2.x - p1.x;
        const dy = p2.y - p1.y;

        // Calculate perpendicular vector n
        const n = { x: -dy, y: dx };

        return {
            point: p1,
            n,
        };
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasLineAB key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new LineABShape(this.dbObject, []);
    }
} 