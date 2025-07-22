import type { Shape, PartialDBObject, PointToLineDistanceInvariantProperties } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { LineABShape } from './LineABShape';
import { CanvasPointToLineDistanceInvariant } from './CanvasComponents';
import { getPointDescription, distanceToLineSegment, parsePoint } from '../utils';

export class PointToLineDistanceInvariantShape extends BaseShape {
    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];

        const properties = dbObject.properties as Partial<PointToLineDistanceInvariantProperties>;
        const point = properties.point;
        const line = properties.line;

        // Parse point
        if (point) {
            const pointCoords = parsePoint(point, shapes);
            if (pointCoords) {
                this.points.push(pointCoords);
            }
        }

        // Parse line and find the perpendicular point
        if (line) {
            // Find the line shape by name
            const lineShape = shapes.find(s => s.dbObject.name === line);
            if (lineShape && lineShape instanceof LineABShape && lineShape.points.length >= 2) {
                const linePoint1 = lineShape.points[0];
                const linePoint2 = lineShape.points[1];

                // Check if our point is already on the line
                if (this.points.length > 0) {
                    const point = this.points[0];

                    // Calculate distance from point to line
                    const distance = this.distanceToLine(point, linePoint1, linePoint2);

                    // If distance is very small (point is on the line), just return the point
                    if (distance < 0.01) {
                        // Point is on the line, no need to add perpendicular point
                    } else {
                        // Calculate the perpendicular point
                        const perpendicularPoint = this.getPerpendicularPoint(point, linePoint1, linePoint2);
                        if (perpendicularPoint) {
                            this.points.push(perpendicularPoint);
                        }
                    }
                }
            }
        }
    }

    private distanceToLine(point: Vector2d, linePoint1: Vector2d, linePoint2: Vector2d): number {
        const A = point.x - linePoint1.x;
        const B = point.y - linePoint1.y;
        const C = linePoint2.x - linePoint1.x;
        const D = linePoint2.y - linePoint1.y;

        const lenSq = C * C + D * D;

        if (lenSq === 0) {
            // Line is actually a point
            return Math.sqrt(A * A + B * B);
        }

        // Calculate perpendicular distance to infinite line
        const lineA = D;  // y2 - y1
        const lineB = -C; // -(x2 - x1)
        const lineC = C * linePoint1.y - D * linePoint1.x; // (x2-x1)y1 - (y2-y1)x1

        return Math.abs(lineA * point.x + lineB * point.y + lineC) / Math.sqrt(lineA * lineA + lineB * lineB);
    }

    private getPerpendicularPoint(point: Vector2d, linePoint1: Vector2d, linePoint2: Vector2d): Vector2d | null {
        const dx = linePoint2.x - linePoint1.x;
        const dy = linePoint2.y - linePoint1.y;
        const lenSq = dx * dx + dy * dy;

        if (lenSq === 0) {
            return null; // Line is actually a point
        }

        // Calculate the parameter t for the perpendicular projection
        // t = ((point - linePoint1) · (linePoint2 - linePoint1)) / |linePoint2 - linePoint1|²
        const t = ((point.x - linePoint1.x) * dx + (point.y - linePoint1.y) * dy) / lenSq;

        // Calculate the perpendicular point
        const perpendicularX = linePoint1.x + t * dx;
        const perpendicularY = linePoint1.y + t * dy;

        return { x: perpendicularX, y: perpendicularY };
    }

    getActionType(): ActionType | null {
        return ActionType.DistanceInvariant;
    }

    getDescription(): string {
        const properties = this.dbObject.properties as Partial<PointToLineDistanceInvariantProperties>;
        const point = getPointDescription(properties.point ?? null);
        const line = properties.line ?? "?";
        return `d(${point}, ${line}) = const`;
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
        return <CanvasPointToLineDistanceInvariant key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new PointToLineDistanceInvariantShape(this.dbObject, []);
    }
} 