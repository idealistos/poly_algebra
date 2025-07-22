import type { Shape, PartialDBObject, TwoLineAngleInvariantProperties } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasTwoLineAngleInvariant } from './CanvasComponents';
import { LineABShape } from './LineABShape';

export class TwoLineAngleInvariantShape extends BaseShape {
    line1Points: Vector2d[];
    line2Points: Vector2d[];

    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        console.log('Called with', dbObject, shapes);
        this.points = [];
        this.line1Points = [];
        this.line2Points = [];
        if (dbObject.properties == null) {
            return;
        }

        const properties = dbObject.properties as Partial<TwoLineAngleInvariantProperties>;
        const line1Name = properties.line1;
        const line2Name = properties.line2;

        // Handle null line names - keep points empty
        if (!line1Name || !line2Name) {
            return;
        }

        // Look up the shapes by name
        const line1Shape = shapes.find(s => s.dbObject.name === line1Name);
        const line2Shape = shapes.find(s => s.dbObject.name === line2Name);

        // Assert that shapes exist and are LineABShape
        if (!line1Shape || !line2Shape) {
            throw new Error(`TwoLineAngleInvariant ${dbObject.name}: Could not find shapes ${line1Name} and/or ${line2Name}`);
        }

        if (!(line1Shape instanceof LineABShape) || !(line2Shape instanceof LineABShape)) {
            throw new Error(`TwoLineAngleInvariant ${dbObject.name}: Shapes ${line1Name} and ${line2Name} must be LineAB shapes`);
        }

        // Calculate intersection point
        const intersectionPoint = line1Shape.intersect(line2Shape);
        if (intersectionPoint === null) {
            throw new Error(`TwoLineAngleInvariant ${dbObject.name}: Lines ${line1Name} and ${line2Name} are parallel and do not intersect`);
        }

        // Add the intersection point to points array
        this.points.push(intersectionPoint);
        this.line1Points = [...line1Shape.points];
        this.line2Points = [...line2Shape.points];
    }

    getActionType(): ActionType | null {
        return ActionType.AngleInvariant;
    }

    getDescription(): string {
        const properties = this.dbObject.properties as Partial<TwoLineAngleInvariantProperties>;
        const line1 = properties.line1 ?? 'undefined';
        const line2 = properties.line2 ?? 'undefined';
        return `α(${line1}, ${line2}) = const`;
    }

    distanceToPoint(point: Vector2d): number {
        return Math.sqrt(
            Math.pow(point.x - this.points[0].x, 2) + Math.pow(point.y - this.points[0].y, 2)
        );
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasTwoLineAngleInvariant key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        const copy = new TwoLineAngleInvariantShape({ ...this.dbObject, properties: null }, []);
        copy.points = this.points;
        copy.line1Points = this.line1Points;
        copy.line2Points = this.line2Points;
        copy.dbObject.properties = this.dbObject.properties;
        return copy;
    }
} 