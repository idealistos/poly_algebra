import type { Shape, PartialDBObject, LocusProperties } from '../types';
import { ObjectType, ActionType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasLocus } from './CanvasComponents';


function getAllowedBaseObjectTypes(): ObjectType[] {
    return [ObjectType.FreePoint, ObjectType.Midpoint, ObjectType.IntersectionPoint];
}

export class LocusShape extends BaseShape {
    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        const point = (dbObject.properties as Partial<LocusProperties>)?.point;
        this.points = [];
        if (point) {
            const locusPoint = shapes.find(s =>
                s.dbObject.name === point &&
                getAllowedBaseObjectTypes().includes(s.dbObject.object_type));
            if (locusPoint) {
                this.points = [locusPoint.getDefinedPoint()!];
            }
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
        return new LocusShape(this.dbObject, []);
    }
} 