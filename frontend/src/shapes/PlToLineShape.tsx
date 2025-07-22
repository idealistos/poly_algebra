import { LineBasedShape } from './LineBasedShape';
import type { PartialDBObject, Shape, PlToLineProperties, Line, CanvasProperties } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { Vector2d } from 'konva/lib/types';
import { parsePoint, getPointDescription } from '../utils';
import { CanvasPlToLine } from './CanvasComponents';

export class PlToLineShape extends LineBasedShape {
    private normal: Vector2d = { x: 0, y: 0 };

    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];

        const properties = dbObject.properties as Partial<PlToLineProperties>;
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

            // Set this.normal to lineShape.getDefinedLine().n (same direction for parallel)
            const lineDef = (lineShape as LineBasedShape).getDefinedLine();
            if (!lineDef) {
                throw new Error(`Line definition not found for: ${line}`);
            }

            this.normal = { ...lineDef.n };
        }
    }

    getActionType(): ActionType | null {
        return ActionType.PlToLine;
    }

    getDescription(): string {
        const properties = this.dbObject.properties as Partial<PlToLineProperties>;
        const point = getPointDescription(properties.point ?? null);
        const line = properties.line ?? 'unknown';
        return `${this.dbObject.name} ∥ ${line} through ${point}`;
    }

    getDefinedLine(): Line | null {
        if (this.points.length === 0) return null;

        return {
            point: this.points[0],
            n: this.normal
        };
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasPlToLine key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        const clone = new PlToLineShape(this.dbObject, []);
        clone.points.push(this.points[0]);
        clone.normal = this.normal;
        return clone;
    }
} 