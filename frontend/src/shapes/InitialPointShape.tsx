import type { Shape, PartialDBObject, LineABProperties, PointToLineDistanceInvariantProperties } from '../types';
import { ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasInitialPoint } from './CanvasComponents';
import { parsePoint } from '../utils';

export class InitialPointShape extends BaseShape {
    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);
        this.points = [];
        let pointStr;
        switch (this.dbObject.object_type) {
            case ObjectType.Midpoint:
            case ObjectType.ScaledVectorPoint:
            case ObjectType.LineAB:
            case ObjectType.TwoPointDistanceInvariant:
                pointStr = (this.dbObject.properties as { point1?: string })?.point1;
                break;
            case ObjectType.PpToLine:
            case ObjectType.PlToLine:
            case ObjectType.PointToLineDistanceInvariant:
            case ObjectType.Projection:
            case ObjectType.Reflection:
                pointStr = (this.dbObject.properties as { point?: string })?.point;
                break;
            default:
                pointStr = null;
        }
        if (pointStr) {
            const point = parsePoint(pointStr, shapes);
            if (point) {
                this.points = [point];
            }
        }
    }

    getActionType(): null {
        return null;
    }

    getDescription(): string {
        return "";
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasInitialPoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    getDBObjectForNextStep(): PartialDBObject | null {
        switch (this.dbObject.object_type) {
            case ObjectType.Midpoint:
            case ObjectType.ScaledVectorPoint:
            case ObjectType.LineAB:
            case ObjectType.PpBisector:
            case ObjectType.TwoPointDistanceInvariant:
                {
                    const point1Str = (this.dbObject.properties as Partial<LineABProperties>)?.point1 ?? this.points[0].x + "," + this.points[0].y;
                    return { ...this.dbObject, properties: { ...this.dbObject.properties, point1: point1Str } as Partial<LineABProperties> };
                }
            case ObjectType.PpToLine:
            case ObjectType.PlToLine:
            case ObjectType.PointToLineDistanceInvariant:
            case ObjectType.Projection:
            case ObjectType.Reflection:
                {
                    const pointStr = (this.dbObject.properties as Partial<PointToLineDistanceInvariantProperties>)?.point ?? this.points[0].x + "," + this.points[0].y;
                    return { ...this.dbObject, properties: { ...this.dbObject.properties, point: pointStr } as Partial<PointToLineDistanceInvariantProperties> };
                }
            case ObjectType.TwoLineAngleInvariant:
                return this.dbObject;
            default:
                return null;
        }
    }

    protected createClone(): Shape {
        const clone = new InitialPointShape(this.dbObject, []);
        clone.points = this.points;
        return clone;
    }
} 