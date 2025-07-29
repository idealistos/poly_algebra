import { BaseShape } from './BaseShape';
import type { Vector2d } from 'konva/lib/types';
import type { CanvasProperties, PartialDBObject, ScaledVectorPointProperties, Shape } from '../types';
import { ActionType } from '../enums';
import { parsePoint } from '../utils';
import { CanvasScaledVectorPoint } from './CanvasComponents';

export class ScaledVectorPointShape extends BaseShape {
    constructor(dbObject: PartialDBObject, shapes: Shape[]) {
        super(dbObject);

        this.points = [];

        const properties = dbObject.properties as Partial<ScaledVectorPointProperties>;
        const point1 = properties.point1;
        const point2 = properties.point2;
        let point1Coords: Vector2d | null = null;
        let point2Coords: Vector2d | null = null;

        // Parse point1
        if (point1) {
            point1Coords = parsePoint(point1, shapes);
            if (point1Coords) {
                this.points.push(point1Coords);
            }
        }

        // Parse point2
        if (point2) {
            point2Coords = parsePoint(point2, shapes);
            if (point2Coords) {
                this.points.push(point2Coords);
            }
        }
        if (point1Coords && point2Coords) {
            const kValue = properties.k_value;
            if (kValue) {
                this.points.push({
                    x: point1Coords.x + kValue * (point2Coords.x - point1Coords.x),
                    y: point1Coords.y + kValue * (point2Coords.y - point1Coords.y)
                })
            }
        }
    }

    getActionType(): ActionType {
        return ActionType.ScaledVectorPoint;
    }

    getDescription(): string {
        const props = this.dbObject.properties as Partial<ScaledVectorPointProperties>;
        let point1 = props.point1 ?? '?';
        let point2 = props.point2 ?? '?';
        const kValue = props.k_value;
        const kValueFormatted = kValue !== undefined ? kValue.toFixed(2) : '?';
        const parts = [`k = ${kValueFormatted}`];
        if (point1.includes(',')) {
            parts.push(`M = (${point1})`);
            point1 = 'M';
        }
        if (point2.includes(',')) {
            parts.push(`N = (${point2})`);
            point2 = 'N';
        }
        return `${this.dbObject.name}: ${point1} + k (${point2} - ${point1}), ${parts.join(', ')}`;
    }

    getDefinedPoint(): Vector2d | null {
        return this.points[2] || null;
    }

    distanceToPoint(point: Vector2d): number {
        if (this.points.length < 3) return Infinity;
        return Math.sqrt(
            Math.pow(point.x - this.points[2].x, 2) + Math.pow(point.y - this.points[2].y, 2)
        );
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasScaledVectorPoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    updatePoints(step: number, point: Vector2d): void {
        if (step !== 2) {
            throw new Error('ScaledVectorPointShape can only be updated at step 2');
        }
        if (!this.points[0]) {
            return;
        }
        console.log('updatePoints', point, this.points);
        this.points[1] = point;
        const kValue = (this.dbObject.properties as Partial<ScaledVectorPointProperties>).k_value ?? 0;
        this.points[2] = {
            x: this.points[0].x + kValue * (point.x - this.points[0].x),
            y: this.points[0].y + kValue * (point.y - this.points[0].y)
        }
        console.log('updatePoints', this);
    }

    createClone(): ScaledVectorPointShape {
        const copy = new ScaledVectorPointShape(this.dbObject, []);
        copy.points = [...this.points];
        return copy;
    }
} 