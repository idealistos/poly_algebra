import type { Shape, PartialDBObject, IntersectionPointProperties } from '../types';
import { ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasIntersectionPoint } from './CanvasComponents';
import { LineABShape } from './LineABShape';

export class IntersectionPointShape extends BaseShape {
    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];
        if (dbObject.properties == null) {
            return;
        }

        const properties = dbObject.properties as Partial<IntersectionPointProperties>;
        const objectName1 = properties.object_name_1;
        const objectName2 = properties.object_name_2;

        // Handle null object names - keep points empty
        if (!objectName1 || !objectName2) {
            return;
        }

        // Look up the shapes by name
        const shape1 = shapes.find(s => s.dbObject.name === objectName1);
        const shape2 = shapes.find(s => s.dbObject.name === objectName2);

        // Assert that shapes exist and are LineABShape
        if (!shape1 || !shape2) {
            throw new Error(`IntersectionPoint ${dbObject.name}: Could not find shapes ${objectName1} and/or ${objectName2}`);
        }

        if (!(shape1 instanceof LineABShape) || !(shape2 instanceof LineABShape)) {
            throw new Error(`IntersectionPoint ${dbObject.name}: Shapes ${objectName1} and ${objectName2} must be LineAB shapes`);
        }

        // Calculate intersection point
        const intersectionPoint = shape1.intersect(shape2);
        if (intersectionPoint === null) {
            throw new Error(`IntersectionPoint ${dbObject.name}: Lines ${objectName1} and ${objectName2} are parallel and do not intersect`);
        }

        // Add the intersection point to points array
        this.points.push(intersectionPoint);
    }

    getActionType(): ActionType | null {
        return ActionType.IntersectionPoint;
    }

    getDescription(): string {
        const properties = this.dbObject.properties as Partial<IntersectionPointProperties>;
        const object1 = properties.object_name_1 ?? 'undefined';
        const object2 = properties.object_name_2 ?? 'undefined';
        return `${this.dbObject.name} (${object1}, ${object2})`;
    }

    getDefinedPoint(): Vector2d | null {
        const properties = this.dbObject.properties as Partial<IntersectionPointProperties>;
        const objectName1 = properties.object_name_1;
        const objectName2 = properties.object_name_2;

        // Verify that object names are non-null
        if (!objectName1 || !objectName2) {
            return null;
        }

        // If we already have the intersection point calculated, return it
        if (this.points.length > 0) {
            return this.points[0];
        }

        // Otherwise, we need to recalculate (this might happen if shapes are updated)
        // For now, return null since we don't have access to the shapes array here
        // The intersection should be calculated in the constructor
        return null;
    }

    distanceToPoint(point: Vector2d): number {
        const definedPoint = this.getDefinedPoint();
        if (!definedPoint) return Infinity;
        return Math.sqrt(
            Math.pow(point.x - definedPoint.x, 2) + Math.pow(point.y - definedPoint.y, 2)
        );
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasIntersectionPoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        const copy = new IntersectionPointShape({ ...this.dbObject, properties: null }, []);
        copy.points = this.points;
        copy.dbObject.properties = this.dbObject.properties;
        return copy;
    }
} 