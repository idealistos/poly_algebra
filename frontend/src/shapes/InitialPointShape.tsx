import type { Shape } from '../types';
import { ObjectType } from '../enums';
import React from 'react';
import type { CanvasProperties } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { CanvasInitialPoint } from './CanvasComponents';

export class InitialPointShape extends BaseShape {
    objectType: ObjectType | null = null;
    point: Vector2d;
    constructor(name: string, point: Vector2d) {
        super(name, "");
        this.point = point;
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

    getCoveredPoints(): { x: number; y: number }[] {
        return [];
    }

    protected createClone(): Shape {
        return new InitialPointShape(this.name, this.point);
    }
} 