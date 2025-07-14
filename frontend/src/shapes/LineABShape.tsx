import type { Shape, PartialDBObject, LineABProperties } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasLineAB } from './CanvasComponents';
import { getPointDescription } from '../utils';

export class LineABShape extends BaseShape {
    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];

        const properties = dbObject.properties as Partial<LineABProperties>;
        const point1 = properties.point1;
        const point2 = properties.point2;

        // Parse point1
        if (point1) {
            const point1Coords = this.parsePoint(point1, shapes);
            if (point1Coords) {
                this.points.push(point1Coords);
            }
        }

        // Parse point2
        if (point2) {
            const point2Coords = this.parsePoint(point2, shapes);
            if (point2Coords) {
                this.points.push(point2Coords);
            }
        }
    }

    private parsePoint(pointValue: string, shapes: Shape[]): { x: number; y: number } | null {
        // Check if it's a coordinate string like "x,y"
        const coordMatch = pointValue.match(/^(-?\d+),\s*(-?\d+)$/);
        if (coordMatch) {
            return {
                x: parseInt(coordMatch[1]),
                y: parseInt(coordMatch[2])
            };
        }

        // Otherwise, look up the object by name in shapes
        const shape = shapes.find(s => s.dbObject.name === pointValue);
        if (shape && shape.points.length > 0) {
            return shape.points[0];
        }

        return null;
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

    distanceToPoint(point: Vector2d): number {
        if (this.points.length < 2) return Infinity;

        const p1 = this.points[0];
        const p2 = this.points[1];

        // Calculate the distance from point to infinite line
        const A = point.x - p1.x;
        const B = point.y - p1.y;
        const C = p2.x - p1.x;
        const D = p2.y - p1.y;

        const lenSq = C * C + D * D;

        if (lenSq === 0) {
            // Line is actually a point
            return Math.sqrt(A * A + B * B);
        }

        // Calculate perpendicular distance to infinite line
        // Formula: |Ax + By + C| / sqrt(A² + B²) where line is Ax + By + C = 0
        // For line through (x1,y1) and (x2,y2): (y2-y1)x - (x2-x1)y + (x2-x1)y1 - (y2-y1)x1 = 0
        // So A = y2-y1, B = -(x2-x1), C = (x2-x1)y1 - (y2-y1)x1
        const lineA = D;  // y2 - y1
        const lineB = -C; // -(x2 - x1)
        const lineC = C * p1.y - D * p1.x; // (x2-x1)y1 - (y2-y1)x1

        return Math.abs(lineA * point.x + lineB * point.y + lineC) / Math.sqrt(lineA * lineA + lineB * lineB);
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

    intersect(line: LineABShape): Vector2d | null {
        if (this.points.length < 2 || line.points.length < 2) {
            return null;
        }

        const p1 = this.points[0];
        const p2 = this.points[1];
        const p3 = line.points[0];
        const p4 = line.points[1];

        // Line 1: through p1 and p2
        // Line 2: through p3 and p4

        // Calculate direction vectors
        const dx1 = p2.x - p1.x;
        const dy1 = p2.y - p1.y;
        const dx2 = p4.x - p3.x;
        const dy2 = p4.y - p3.y;

        // Calculate determinant
        const det = dx1 * dy2 - dy1 * dx2;

        // If determinant is 0, lines are parallel
        if (Math.abs(det) < 1e-10) {
            return null;
        }

        // Calculate intersection point using parametric line equations
        // Line 1: p1 + t1 * (p2 - p1)
        // Line 2: p3 + t2 * (p4 - p3)
        // At intersection: p1 + t1 * (p2 - p1) = p3 + t2 * (p4 - p3)

        const dx3 = p3.x - p1.x;
        const dy3 = p3.y - p1.y;

        const t1 = (dx3 * dy2 - dy3 * dx2) / det;

        // Calculate intersection point
        const intersectionX = p1.x + t1 * dx1;
        const intersectionY = p1.y + t1 * dy1;

        return { x: intersectionX, y: intersectionY };
    }
} 