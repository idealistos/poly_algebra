import type { Shape, PointToLineDistanceInvariantProperties, Line, ObjectProperties, ShapeCreatorInput, ArgumentValue } from '../types';
import { ActionType, ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape, BaseShapeCreator } from './BaseShape';
import { CanvasPointToLineDistanceInvariant } from './CanvasComponents';
import { getShapeNameOrPoint, getDefinedOrGridPoint, getPointsFromInput, distanceToLineSegment } from '../utils';
import { InitialPointShape } from './InitialPointShape';
import { LineBasedShape } from './LineBasedShape';
import { ProjectionShape } from './ProjectionShape';

export class PointToLineDistanceInvariantShape extends BaseShape {
    objectType: ObjectType = ObjectType.PointToLineDistanceInvariant;
    point: Vector2d;
    line: Line;
    perpendicularPoint: Vector2d | null;

    constructor(name: string, description: string, point: Vector2d, line: Line) {
        super(name, description);
        this.point = point;
        this.line = line;
        this.perpendicularPoint = null;

        // Parse line and find the perpendicular point
        const linePoint1 = line.point;
        const linePoint2 = { x: line.point.x + line.n.y, y: line.point.y - line.n.x };

        // Calculate distance from point to line
        const distance = this.distanceToLine(point, linePoint1, linePoint2);

        // If distance is very small (point is on the line), just return the point
        if (distance < 0.01) {
            // Point is on the line, no need to add perpendicular point
        } else {
            // Calculate the perpendicular point
            this.perpendicularPoint = this.getPerpendicularPoint(point, linePoint1, linePoint2);
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

    getCoveredPoints(): { x: number; y: number }[] {
        return [];
    }

    distanceToPoint(point: Vector2d): number {
        if (this.perpendicularPoint == null) {
            return Infinity;
        }
        return distanceToLineSegment(point, this.point, this.perpendicularPoint);
    }


    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasPointToLineDistanceInvariant key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new PointToLineDistanceInvariantShape(this.name, this.description, this.point, this.line);
    }
}

export class PointToLineDistanceInvariantShapeCreator extends BaseShapeCreator {
    objectType: ObjectType = ObjectType.PointToLineDistanceInvariant;

    getDBObjectProperties(input: ShapeCreatorInput): ObjectProperties {
        return {
            point: getShapeNameOrPoint(input.argumentValues[0]?.[0]),
            line: (input.argumentValues[1]?.[0] as Shape).name,
        };
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const pointToLineDistanceProperties = properties as PointToLineDistanceInvariantProperties;
        const point = getDefinedOrGridPoint(pointToLineDistanceProperties.point, shapes);
        const line = shapes.find(shape => shape.name === pointToLineDistanceProperties.line);
        if (point == null || line == null) {
            throw new Error('Invalid point or line value');
        }
        return [[point], [line]];
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        const points = getPointsFromInput(input);
        const lineShape = input.argumentValues[1]?.[0] as LineBasedShape;
        const line = lineShape?.getDefinedLine() || null;

        if (points.length == 0) {
            return null;
        } else if (points.length == 1) {
            if (line == null) {
                return new InitialPointShape(input.objectName, points[0]);
            }
            return new PointToLineDistanceInvariantShape(input.objectName, this.getDescription(input), points[0], line);
        } else {
            // Hinted
            return new ProjectionShape(input.objectName, "", points[0], points[1], null);
        }
    }

    protected getDescriptionInner(input: ShapeCreatorInput, argumentStringValues: string[]): string {
        const lineName = (input.argumentValues[1]?.[0] as Shape)?.name || '?';
        return `d(${argumentStringValues[0]}, ${lineName}) = const`;
    }
} 