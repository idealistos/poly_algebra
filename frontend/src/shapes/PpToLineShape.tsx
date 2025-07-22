import { LineBasedShape } from './LineBasedShape';
import type { PartialDBObject, Shape, PpToLineProperties, Line, CanvasProperties } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { Vector2d } from 'konva/lib/types';
import { parsePoint, getPointDescription } from '../utils';
import { CanvasPpToLine } from './CanvasComponents';

export class PpToLineShape extends LineBasedShape {
    private normal: Vector2d = { x: 0, y: 0 };

    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];

        const properties = dbObject.properties as Partial<PpToLineProperties>;
        const point = properties.point;
        const line = properties.line;

        const pointCoords = point ? parsePoint(point, shapes) : null;
        if (pointCoords == null) {
            return;
        }

        if (line == null) {
            this.points = [pointCoords];
            return;
        }

        // Find lineShape via "shapes"
        const lineShape = shapes.find(shape => shape.dbObject.name === line);
        if (lineShape) {

            // Set this.normal to lineShape.getDefinedLine().n rotated by 90 degrees
            const lineDef = (lineShape as LineBasedShape).getDefinedLine();
            if (!lineDef) {
                throw new Error(`Line definition not found for: ${line}`);
            }

            this.normal = {
                x: -lineDef.n.y,
                y: lineDef.n.x
            };

            // Find the point on lineShape closest to the one represented by dbObject.properties.point
            const closestPoint = this.findClosestPointOnLine(pointCoords, lineDef);
            this.points = [closestPoint];
        }
    }

    getActionType(): ActionType | null {
        return ActionType.PpToLine;
    }

    getDescription(): string {
        const properties = this.dbObject.properties as Partial<PpToLineProperties>;
        const point = getPointDescription(properties.point ?? null);
        const line = properties.line ?? 'unknown';
        return `${this.dbObject.name} ⟂ ${line} through ${point}`;
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
        return <CanvasPpToLine key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        const clone = new PpToLineShape(this.dbObject, []);
        clone.points.push(this.points[0]);
        clone.normal = this.normal;
        return clone;
    }

    private findClosestPointOnLine(point: Vector2d, lineDef: Line): Vector2d {
        // Calculate the closest point on the line to the given point
        // This is the projection of the point onto the line
        const linePoint = lineDef.point;
        const lineNormal = lineDef.n;

        // Vector from line point to the given point
        const vectorToPoint = {
            x: point.x - linePoint.x,
            y: point.y - linePoint.y
        };

        // Project the vector onto the line normal
        const projection = (vectorToPoint.x * lineNormal.x + vectorToPoint.y * lineNormal.y) /
            (lineNormal.x * lineNormal.x + lineNormal.y * lineNormal.y);

        // The closest point is the line point plus the projection
        return {
            x: point.x - projection * lineNormal.x,
            y: point.y - projection * lineNormal.y
        };
    }
} 