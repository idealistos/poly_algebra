import type { PartialDBObject, Line } from '../types';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './BaseShape';

export abstract class LineBasedShape extends BaseShape {
    constructor(dbObject: PartialDBObject) {
        super(dbObject);
    }

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

    intersect(line: LineBasedShape): Vector2d | null {
        const thisLine = this.getDefinedLine();
        const otherLine = line.getDefinedLine();

        if (!thisLine || !otherLine) return null;

        // Solve the system of equations:
        // n1 · (p - point1) = 0
        // n2 · (p - point2) = 0

        // This gives us:
        // n1x * (px - point1x) + n1y * (py - point1y) = 0
        // n2x * (px - point2x) + n2y * (py - point2y) = 0

        // Rearranging:
        // n1x * px + n1y * py = n1x * point1x + n1y * point1y
        // n2x * px + n2y * py = n2x * point2x + n2y * point2y

        const a11 = thisLine.n.x;
        const a12 = thisLine.n.y;
        const a21 = otherLine.n.x;
        const a22 = otherLine.n.y;

        const b1 = thisLine.n.x * thisLine.point.x + thisLine.n.y * thisLine.point.y;
        const b2 = otherLine.n.x * otherLine.point.x + otherLine.n.y * otherLine.point.y;

        // Calculate determinant
        const det = a11 * a22 - a12 * a21;

        if (Math.abs(det) < 1e-10) {
            // Lines are parallel
            return null;
        }

        // Solve using Cramer's rule
        const px = (b1 * a22 - b2 * a12) / det;
        const py = (a11 * b2 - a21 * b1) / det;

        return { x: px, y: py };
    }
} 