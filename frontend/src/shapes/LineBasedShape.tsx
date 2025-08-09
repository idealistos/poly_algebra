import type { Line } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';
import { ObjectType } from '../enums';
import { intersectLines } from '../utils';

export abstract class LineBasedShape extends BaseShape {
    abstract getDefinedLine(): Line | null;

    distanceToPoint(point: Vector2d): number {
        const line = this.getDefinedLine();
        if (!line) return Infinity;

        // Calculate the distance from point to line
        // Line equation: n · (p - point) = 0
        // Distance = |n · (p - point)| / |n|
        const dx = point.x - line.point.x;
        const dy = point.y - line.point.y;

        const dotProduct = line.n.x * dx + line.n.y * dy;
        const nMagnitude = Math.sqrt(line.n.x * line.n.x + line.n.y * line.n.y);

        if (nMagnitude === 0) return Infinity;

        return Math.abs(dotProduct) / nMagnitude;
    }

    getCoveredPoints(): { x: number; y: number }[] {
        return [];
    }

    matchesLastArgumentOf(objectType: ObjectType): boolean {
        return objectType === ObjectType.PointToLineDistanceInvariant;
    }

    intersect(line: LineBasedShape): Vector2d | null {
        const thisLine = this.getDefinedLine();
        const otherLine = line.getDefinedLine();
        if (thisLine == null || otherLine == null) {
            return null;
        }
        return intersectLines(thisLine, otherLine);
    }
} 