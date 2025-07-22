import type { Shape, PartialDBObject, LocusProperties } from '../types';
import { ObjectType, ActionType, MOBILE_POINT_OBJECT_TYPES } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasLocus } from './CanvasComponents';


export class LocusShape extends BaseShape {
    public locusOrdinal: number;

    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        const point = (dbObject.properties as Partial<LocusProperties>)?.point;
        this.points = [];
        if (point) {
            const locusPoint = shapes.find(s =>
                s.dbObject.name === point &&
                MOBILE_POINT_OBJECT_TYPES.includes(s.dbObject.object_type));
            if (locusPoint) {
                this.points = [locusPoint.getDefinedPoint()!];
            }
        }

        // Calculate locus ordinal based on existing locus shapes
        const existingLocusShapes = shapes.filter(s => s.dbObject.object_type === ObjectType.Locus);
        const matchingShape = existingLocusShapes.find(s => s.dbObject.name === dbObject.name);

        if (matchingShape) {
            // If this shape already exists, use its index
            this.locusOrdinal = existingLocusShapes.indexOf(matchingShape);
        } else {
            // If this is a new shape, use the count of existing locus shapes
            this.locusOrdinal = existingLocusShapes.length;
        }
    }

    getActionType(): ActionType | null {
        return ActionType.Locus;
    }

    getDescription(): string {
        const point = (this.dbObject.properties as Partial<LocusProperties>)?.point ?? '?';
        return `Plot {${point}}`;
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasLocus key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        const clone = new LocusShape(this.dbObject, []);
        clone.locusOrdinal = this.locusOrdinal;
        return clone;
    }
} 