import type { Vector2d } from "konva/lib/types";
import { BaseShape } from "./BaseShape";
import { ObjectType } from "../enums";

export abstract class PointBasedShape extends BaseShape {
    distanceToPoint(point: Vector2d): number {
        const definedPoint = this.getDefinedPoint();
        if (definedPoint == null) {
            return Infinity;
        }
        return Math.sqrt(
            Math.pow(point.x - definedPoint.x, 2) + Math.pow(point.y - definedPoint.y, 2)
        );
    }

    getCoveredPoints(): { x: number; y: number }[] {
        const definedPoint = this.getDefinedPoint();
        if (definedPoint == null) {
            return [];
        }
        return [definedPoint];
    }

    matchesLastArgumentOf(objectType: ObjectType): boolean {
        return objectType === ObjectType.TwoPointDistanceInvariant;
    }
}