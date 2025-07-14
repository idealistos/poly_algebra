import type { Shape, PartialDBObject, LineABProperties } from '../types';
import { ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasInitialPoint } from './CanvasComponents';

export class InitialPointShape extends BaseShape {
    constructor(dbObject: PartialDBObject) {
        super(dbObject);
        console.log("InitialPointShape", dbObject);
        this.points = [{ x: 0, y: 0 }];
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
            case ObjectType.LineAB: {
                const point1Str = (this.dbObject.properties as Partial<LineABProperties>)?.point1 ?? this.points[0].x + "," + this.points[0].y;
                return { ...this.dbObject, properties: { ...this.dbObject.properties, point1: point1Str } as Partial<LineABProperties> };
            }
            default:
                return null;
        }
    }

    protected createClone(): Shape {
        return new InitialPointShape(this.dbObject);
    }
} 