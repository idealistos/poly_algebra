import { type Action, type Shape, type ArgumentValue, type ShapeCreatorInput, type PartialDBObject, argValuesAreEqual } from './types';
import { ShapeState, ArgumentType, ObjectType } from './enums';
import type { Vector2d } from 'konva/lib/types';
import { BaseShape } from './shapes/BaseShape';
import { getClosestDefinedPoint, getClosestLine, getMatchingObjectTypes, getShapeCreator, getTwoClosestLines } from './utils';

export class Stage {
    action: Action;
    private shapes: Shape[];
    private stagedShapeName: string;
    currentStep: number;
    private validatedExpressions: string[];
    private expressionValues: number[];
    inputArguments: ArgumentValue[];
    hintedObjectPoint: Vector2d | null;
    locusOrdinal: number | null;

    constructor(
        action: Action,
        shapes: Shape[]
    ) {
        this.action = action;
        this.shapes = shapes;
        this.stagedShapeName = computeStagedShapeName(shapes, action);
        this.currentStep = 0;
        this.validatedExpressions = [];
        this.expressionValues = [];
        this.inputArguments = [];
        this.hintedObjectPoint = null;
        this.locusOrdinal = null;
        if (this.action.name === "Locus") {
            this.locusOrdinal = this.shapes.filter(s => s.objectType === ObjectType.Locus).length;
        }
    }

    setHintedPoint(point: { x: number, y: number }): boolean {
        if (this.hintedObjectPoint != null && this.hintedObjectPoint.x === point.x && this.hintedObjectPoint.y === point.y) {
            return false;
        }
        this.hintedObjectPoint = point;
        delete this.inputArguments[this.currentStep];
        return true;
    }

    addValidatedExpression(expression: string, value: number): void {
        this.validatedExpressions.push(expression);
        this.expressionValues.push(value);
        this.currentStep++;
    }

    setStagedArgument(argValue: ArgumentValue): boolean {
        if (argValuesAreEqual(this.inputArguments[this.currentStep], argValue)) {
            return false;
        }
        this.inputArguments[this.currentStep] = argValue;
        this.hintedObjectPoint = null;
        return true;
    }

    canConfirmStagedArgument(): boolean {
        return !this.isCurrentArgumentAnExpression() &&
            this.inputArguments[this.currentStep] != null;
    }

    confirmStagedArgument(): void {
        this.currentStep++;
    }

    getActionTitle(): string {
        return this.action.description.split(":")[0];
    }

    getArgumentHint(): string {
        return this.action.arguments[this.currentStep]?.hint ?? '';
    }

    getCurrentArgumentTypes(): ArgumentType[] {
        if (this.isReady()) {
            return [];
        }
        return this.action.arguments[this.currentStep].types;
    }

    isCurrentArgumentAnExpression(): boolean {
        return !this.isReady() && this.getCurrentArgumentTypes().length === 0;
    }

    getExclusiveObjectTypesForCurrentArgument(): ObjectType[] {
        if (this.isReady()) {
            return [];
        }
        return this.action.arguments[this.currentStep].exclusive_object_types;
    }

    getObjectTypeByArgumentTypeIndex(argumentTypeIndex: number): ObjectType {
        if (this.currentStep != this.action.arguments.length - 1) {
            return this.action.object_types[0];
        }
        return this.action.object_types[argumentTypeIndex];
    }

    getShape(): Shape | null {
        // If isReady() returns false: "staged" or "hinted", depending on hintedObjectPoint
        // if isReady() returns true: "Default"
        const input = this.getShapeCreatorInput();
        const shapeCreator = getShapeCreator(this.getObjectType());
        const shape = shapeCreator.createShape(input);
        if (shape == null) {
            return null;
        }
        console.log("shape", shape);
        if (this.isReady()) {
            shape.state = ShapeState.Default;
        } else if (this.inputArguments[this.currentStep] != null) {
            shape.state = ShapeState.BeingAdded;
        } else {
            shape.state = ShapeState.Hinted;
        }
        return shape;
    }

    isReady(): boolean {
        return this.currentStep >= this.action.arguments.length;
    }

    getObjectType(): ObjectType {
        const lastArgument = this.inputArguments[this.action.arguments.length - 1]?.[0];
        if (this.action.object_types.length === 1 || lastArgument == null) {
            return this.action.object_types[0];
        }
        const argumentTypes = this.action.arguments[this.action.arguments.length - 1].types;
        for (let i = 0; i < argumentTypes.length; ++i) {
            if (lastArgument instanceof BaseShape) {
                if (lastArgument.matchesLastArgumentOf(this.action.object_types[i])) {
                    return this.action.object_types[i];
                }
            } else {
                // lastArgument is vector2d (grid point)
                if (this.action.object_types[i] === ObjectType.TwoPointDistanceInvariant) {
                    return this.action.object_types[i];
                }
            }
        }
        throw new Error(`${JSON.stringify(lastArgument, null, 2)} cannot serve as the last argument for the action ${this.action.name}`);
    }

    getDBObject(): PartialDBObject {
        const input = this.getShapeCreatorInput();
        const shapeCreator = getShapeCreator(this.getObjectType());
        const properties = shapeCreator.getDBObjectProperties(input);
        return {
            name: this.stagedShapeName,
            object_type: this.getObjectType(),
            properties,
        };
    }

    getOccupiedPoints(): Vector2d[] {
        return this.inputArguments.filter(argValue => argValue instanceof BaseShape)
            .map(argValue => argValue.getDefinedPoint()).filter(point => point != null);
    }

    getArgumentValueForCoordinates(
        logicalPoint: Vector2d,
    ): { argValue: ArgumentValue, shapesToHighlight: string[], objectType: ObjectType } | null {
        const pointsOccupiedByPartialObject = this.getOccupiedPoints();
        const isOccupied = (point: Vector2d) => this.shapes.some(
            s => {
                if (this.getExclusiveObjectTypesForCurrentArgument().includes(s.objectType!)) {
                    const definedPoint = s.getDefinedPoint();
                    return definedPoint != null &&
                        definedPoint.x === point.x &&
                        definedPoint.y === point.y;
                }
                return false;
            }
        ) || pointsOccupiedByPartialObject.some(p => p.x === point.x && p.y === point.y);
        for (const [index, argumentType] of this.getCurrentArgumentTypes().entries()) {
            const argumentObjectType = this.getObjectTypeByArgumentTypeIndex(index);
            // Handle point-related argument types

            switch (argumentType) {
                case ArgumentType.GridPoint:
                case ArgumentType.MobilePoint:
                case ArgumentType.AnyDefinedPoint:
                case ArgumentType.AnyDefinedOrGridPoint: {
                    const result = this.getPointArgumentValue(logicalPoint, argumentType, argumentObjectType, isOccupied);
                    if (result != null) {
                        return result;
                    }
                    break;
                }
                case ArgumentType.IntersectionPoint: {
                    const twoClosestLines = getTwoClosestLines(this.shapes, logicalPoint);
                    console.log('twoClosestLines', twoClosestLines);
                    if (twoClosestLines == null) {
                        break;
                    }
                    const [line1, line2] = twoClosestLines;
                    if (line1.distance > 0.15 || line2.distance > 0.15) {
                        break;
                    }
                    const intersectionPoint = line1.shape.intersect(line2.shape);
                    if (intersectionPoint == null || isOccupied(intersectionPoint)) {
                        break;
                    }
                    const distToIntersection = Math.sqrt(
                        Math.pow(logicalPoint.x - intersectionPoint.x, 2) +
                        Math.pow(logicalPoint.y - intersectionPoint.y, 2)
                    );
                    if (distToIntersection < 0.25) {
                        return {
                            argValue: [line1.shape, line2.shape],
                            shapesToHighlight: [line1.shape.name, line2.shape.name],
                            objectType: argumentObjectType,
                        };
                    }
                    break;
                }
                case ArgumentType.SlidingPoint: {
                    const gridX = Math.round(logicalPoint.x);
                    const gridY = Math.round(logicalPoint.y);
                    const dist = Math.sqrt(
                        Math.pow(logicalPoint.x - gridX, 2) + Math.pow(logicalPoint.y - gridY, 2)
                    );
                    if (dist > 0.15 || isOccupied({ x: gridX, y: gridY })) {
                        break;
                    }
                    const closestLine = getClosestLine(this.shapes, logicalPoint);
                    if (!closestLine || closestLine.distance > 0.15) {
                        break;
                    }
                    return {
                        argValue: [{ x: gridX, y: gridY }, closestLine.shape],
                        shapesToHighlight: [closestLine.shape.name],
                        objectType: argumentObjectType,
                    }
                }
                case ArgumentType.Line: {
                    const closestLine = getClosestLine(this.shapes, logicalPoint);
                    if (!closestLine || closestLine.distance > 0.15 || this.alreadyHasArgument(closestLine.shape)) {
                        break;
                    }
                    return {
                        argValue: [closestLine.shape],
                        shapesToHighlight: [closestLine.shape.name],
                        objectType: argumentObjectType,
                    };
                }
                default: {
                    const exhaustiveCheck: never = argumentType;
                    throw new Error(`Unhandled argument type: ${exhaustiveCheck}`);
                }
            }
        }
        return null;
    }

    private getShapeCreatorInput(): ShapeCreatorInput {
        return {
            objectName: this.stagedShapeName,
            validatedExpressions: this.validatedExpressions,
            expressionValues: this.expressionValues,
            argumentValues: this.inputArguments,
            hintedObjectPoint: this.hintedObjectPoint,
            locusOrdinal: this.locusOrdinal,
        }
    }

    private getPointArgumentValue(
        logicalPoint: Vector2d,
        argumentType: ArgumentType,
        argumentObjectType: ObjectType,
        isOccupied: (point: Vector2d) => boolean,
    ): { argValue: ArgumentValue, shapesToHighlight: string[], objectType: ObjectType } | null {

        // Helper function to check grid point
        const checkGridPoint = () => {
            const gridX = Math.round(logicalPoint.x);
            const gridY = Math.round(logicalPoint.y);
            const dist = Math.sqrt(
                Math.pow(logicalPoint.x - gridX, 2) + Math.pow(logicalPoint.y - gridY, 2)
            );
            if (dist < 0.15 && !isOccupied({ x: gridX, y: gridY })) {
                return {
                    argValue: [{ x: gridX, y: gridY }],
                    shapesToHighlight: [],
                    objectType: argumentObjectType,
                };
            }
            return null;
        };

        // Helper function to check defined point
        const checkDefinedPoint = (objectTypes: ObjectType[]) => {
            const { shape: closest, minDist } = getClosestDefinedPoint(objectTypes, this.shapes, logicalPoint, isOccupied);
            if (closest != null && minDist < 0.15) {
                return {
                    argValue: [closest],
                    shapesToHighlight: [closest.name],
                    objectType: argumentObjectType,
                };
            }
            return null;
        };

        switch (argumentType) {
            case ArgumentType.GridPoint: {
                return checkGridPoint();
            }
            case ArgumentType.MobilePoint:
            case ArgumentType.AnyDefinedPoint: {
                const objectTypes = getMatchingObjectTypes(argumentType);
                return checkDefinedPoint(objectTypes);
            }
            case ArgumentType.AnyDefinedOrGridPoint: {
                // First try to find a defined point
                const objectTypes = getMatchingObjectTypes(ArgumentType.AnyDefinedPoint);
                const definedResult = checkDefinedPoint(objectTypes);
                if (definedResult) {
                    return definedResult;
                }

                // If no defined point found, try grid point
                return checkGridPoint();
            }
            case ArgumentType.Line: {
                // Line argument type is handled in the main switch statement
                return null;
            }
        }
        return null;
    }

    private alreadyHasArgument(shape: Shape): boolean {
        return this.inputArguments.some(
            (argValue, index) => argValue.some(arg => arg === shape) && index < this.currentStep,
        );
    }


}

function computeStagedShapeName(shapes: Shape[], action: Action): string {
    const existingNames = new Set(shapes.map(s => s.name));
    // Try allowed_names directly
    for (const name of action.allowed_names) {
        if (!existingNames.has(name)) {
            return name;
        }
    }
    // Try allowed_names with suffixes
    for (let suffix = 1; suffix < 100; ++suffix) {
        for (const name of action.allowed_names) {
            const candidate = name + suffix;
            if (!existingNames.has(candidate)) {
                return candidate;
            }
        }
    }

    throw new Error('No available name found');
}
