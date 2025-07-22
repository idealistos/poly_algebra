import { LineBasedShape } from './LineBasedShape';
import type { PartialDBObject, Shape, Line, CanvasProperties, ReflectionProperties } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import { BaseShape } from './BaseShape';
import { getPointDescription, parsePoint } from '../utils';
import type { Vector2d } from 'konva/lib/types';
import { CanvasReflection } from './CanvasComponents';

export class ReflectionShape extends BaseShape {

    private line: Line | null = null;

    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];

        const properties = dbObject.properties as Partial<ReflectionProperties>;
        const point = properties.point;
        const line = properties.line;

        const pointCoords = point ? parsePoint(point, shapes) : null;
        if (pointCoords == null) {
            return;
        }
        this.points = [pointCoords];

        if (line == null) {
            return;
        }

        // Find lineShape via "shapes"
        const lineShape = shapes.find(shape => shape.dbObject.name === line);
        if (lineShape) {

            this.line = (lineShape as LineBasedShape).getDefinedLine();
            if (!this.line) {
                throw new Error(`Line definition not found for: ${line}`);
            }

            const reflectedPoint = this.findReflectedPoint(pointCoords, this.line);
            this.points.push(reflectedPoint);
        }
    }

    getActionType(): ActionType {
        return ActionType.Reflection;
    }

    getDescription(): string {
        const properties = this.dbObject.properties as Partial<ReflectionProperties>;
        const point = getPointDescription(properties.point ?? null);
        return `${this.dbObject.name}: reflection(${point}, ${properties.line || '???'})`;
    }

    getDefinedPoint(): Vector2d | null {
        return this.points[1] || null;
    }

    distanceToPoint(point: Vector2d): number {
        if (this.points.length === 0) return Infinity;
        return Math.sqrt(
            Math.pow(point.x - this.points[1].x, 2) + Math.pow(point.y - this.points[1].y, 2)
        );
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasReflection key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        const clone = new ReflectionShape(this.dbObject, []);
        clone.points = this.points.map(point => ({ ...point }));
        clone.line = this.line;
        return clone;
    }

    private findReflectedPoint(point: Vector2d, line: Line): Vector2d {
        // Vector from line point to the point to reflect
        const dx = point.x - line.point.x;
        const dy = point.y - line.point.y;

        // Dot product: n · (point - line.point)
        const dotProduct = line.n.x * dx + line.n.y * dy;

        // Magnitude squared of normal vector: |n|²
        const nMagnitudeSquared = line.n.x * line.n.x + line.n.y * line.n.y;

        if (nMagnitudeSquared === 0) {
            // Line has zero normal vector, return the original point
            return { x: point.x, y: point.y };
        }

        // Calculate the reflection
        // reflection = point - 2 * (dotProduct / |n|²) × n
        const scale = dotProduct / nMagnitudeSquared;
        const reflectedX = point.x - 2 * scale * line.n.x;
        const reflectedY = point.y - 2 * scale * line.n.y;

        return { x: reflectedX, y: reflectedY };
    }
} 