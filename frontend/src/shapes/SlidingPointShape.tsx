import type { Shape, PartialDBObject, SlidingPointProperties } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasSlidingPoint } from './CanvasComponents';
import { LineBasedShape } from './LineBasedShape';

export class SlidingPointShape extends BaseShape {
    public lineDirection: Vector2d;

    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];

        // Default to horizontal direction if no properties
        if (!dbObject.properties) {
            this.lineDirection = { x: 1, y: 0 };
            return;
        }

        const properties = dbObject.properties as Partial<SlidingPointProperties>;
        const value = properties.value;
        const objectName = properties.constraining_object_name;

        // Parse the value to get coordinates
        if (value) {
            const coords = this.parseCoordinates(value);
            if (coords) {
                this.points.push(coords);
            }
        }

        // Compute line direction based on the line the point slides on
        if (objectName) {
            // Look up the shape by name
            const lineShape = shapes.find(s => s.dbObject.name === objectName);

            // Assert that shape exists and is LineABShape
            if (!lineShape) {
                throw new Error(`SlidingPoint ${dbObject.name}: Could not find line shape ${objectName}`);
            }

            if (!(lineShape instanceof LineBasedShape)) {
                throw new Error(`SlidingPoint ${dbObject.name}: Shape ${objectName} must be a LineAB shape`);
            }
            const lineDef = lineShape.getDefinedLine();
            this.lineDirection = lineDef == null ? { x: 1, y: 0 } : { x: lineDef.n.y, y: -lineDef.n.x };
        } else {
            // Default to horizontal direction if no line name
            this.lineDirection = { x: 1, y: 0 };
        }
    }

    private parseCoordinates(value: string): { x: number; y: number } | null {
        const match = value.match(/^(-?\d+),\s*(-?\d+)$/);
        if (match) {
            return {
                x: parseInt(match[1]),
                y: parseInt(match[2])
            };
        }
        return null;
    }

    getActionType(): ActionType | null {
        return ActionType.SlidingPoint;
    }

    getDescription(): string {
        const properties = this.dbObject.properties as Partial<SlidingPointProperties>;
        const objectName = properties.constraining_object_name ?? 'undefined';
        const value = properties.value ?? 'undefined';
        return `${this.dbObject.name} (${value}) on ${objectName}`;
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
        return <CanvasSlidingPoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        const copy = new SlidingPointShape({ ...this.dbObject, properties: null }, []);
        copy.points = this.points;
        copy.dbObject.properties = this.dbObject.properties;
        copy.lineDirection = this.lineDirection;
        return copy;
    }
} 