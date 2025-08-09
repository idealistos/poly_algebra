import { BaseShapeCreator } from './BaseShape';
import type { Vector2d } from 'konva/lib/types';
import type { CanvasProperties, ScaledVectorPointProperties, Shape, ShapeCreatorInput, ArgumentValue, ObjectProperties, DBObject } from '../types';
import { ActionType, ObjectType } from '../enums';
import { getShapeNameOrPoint, getPointsFromInput, getDefinedOrGridPoint } from '../utils';
import { CanvasScaledVectorPoint } from './CanvasComponents';
import { PointBasedShape } from './PointBasedShape';
import { InitialPointShape } from './InitialPointShape';

export class ScaledVectorPointShape extends PointBasedShape {
    objectType: ObjectType = ObjectType.ScaledVectorPoint;
    point1: Vector2d;
    point2: Vector2d;
    point: Vector2d;
    kValue: number;

    constructor(name: string, description: string, point1: Vector2d, point2: Vector2d, kValue: number) {
        super(name, description);
        this.point1 = point1;
        this.point2 = point2;
        this.kValue = kValue;
        this.point = {
            x: point1.x + kValue * (point2.x - point1.x),
            y: point1.y + kValue * (point2.y - point1.y)
        }
    }

    getActionType(): ActionType | null {
        return ActionType.ScaledVectorPoint;
    }

    getDefinedPoint(): Vector2d | null {
        return this.point;
    }

    getCanvasShape(canvasProperties: CanvasProperties, key?: string): React.ReactNode {
        const getPhysicalCoords = (coords: Vector2d) => ({
            px: (coords.x - canvasProperties.topLeft.x) / canvasProperties.scale,
            py: (canvasProperties.topLeft.y - coords.y) / canvasProperties.scale
        });
        return <CanvasScaledVectorPoint key={key} shape={this} getPhysicalCoords={getPhysicalCoords} />;
    }

    protected createClone(): Shape {
        return new ScaledVectorPointShape(this.name, this.description, this.point1, this.point2, this.kValue);
    }
}

export class ScaledVectorPointShapeCreator extends BaseShapeCreator {
    objectType = ObjectType.ScaledVectorPoint;

    getDBObjectProperties(input: ShapeCreatorInput): ScaledVectorPointProperties {
        if (input.validatedExpressions[0] == null || input.expressionValues[0] == null || input.argumentValues[1]?.[0] == null || input.argumentValues[2]?.[0] == null) {
            throw new Error('Invalid input');
        }
        return {
            k: input.validatedExpressions[0],
            point1: getShapeNameOrPoint(input.argumentValues[1]?.[0]),
            point2: getShapeNameOrPoint(input.argumentValues[2]?.[0]),
            k_value: input.expressionValues[0]
        };
    }

    getInputForDBObject(dbObject: DBObject, shapes: Shape[]): ShapeCreatorInput {
        const properties = dbObject.properties as ScaledVectorPointProperties;
        return {
            objectName: dbObject.name,
            validatedExpressions: [properties.k],
            expressionValues: [properties.k_value],
            argumentValues: this.getArgumentValues(properties, shapes),
            hintedObjectPoint: null,
            locusOrdinal: null
        }
    }

    getArgumentValues(properties: ObjectProperties, shapes: Shape[]): ArgumentValue[] {
        const scaledVectorPointProperties = properties as ScaledVectorPointProperties;
        const point1 = getDefinedOrGridPoint(scaledVectorPointProperties.point1, shapes);
        const point2 = getDefinedOrGridPoint(scaledVectorPointProperties.point2, shapes);
        if (point1 == null || point2 == null) {
            throw new Error('Invalid point1 or point2 value');
        }
        return [[], [point1], [point2]];
    }

    createShape(input: ShapeCreatorInput): Shape | null {
        const points = getPointsFromInput(input);
        if (points.length == 0) {
            return null;
        } else if (points.length == 1) {
            return new InitialPointShape(input.objectName, points[0]);
        }

        const point1 = points[0];
        const point2 = points[1];
        const kValue = input.expressionValues[0] ?? 0;

        return new ScaledVectorPointShape(
            input.objectName,
            this.getDescription(input),
            point1,
            point2,
            kValue,
        );
    }

    protected getDescriptionInner(input: ShapeCreatorInput, argumentStringValues: string[]): string {
        const kValue = input.expressionValues[0] ?? 0;
        const kValueFormatted = kValue.toFixed(2);
        let point1 = argumentStringValues[1];
        let point2 = argumentStringValues[2];
        const parts = [`k = ${kValueFormatted}`];
        if (point1.includes(',')) {
            parts.push(`M = ${point1}`);
            point1 = 'M';
        }
        if (point2.includes(',')) {
            parts.push(`N = ${point2}`);
            point2 = 'N';
        }
        return `${input.objectName}: ${point1} + k (${point2} - ${point1}), ${parts.join(', ')}`;
    }
} 