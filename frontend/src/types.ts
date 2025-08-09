import { ActionGroup, ActionType, ArgumentType, ObjectType, ShapeState } from './enums';
import type { Vector2d } from 'konva/lib/types';

export interface ActionArgument {
  types: ArgumentType[];
  hint: string;
  exclusive_object_types: ObjectType[];
}

export interface Action {
  name: ActionType;
  object_types: ObjectType[];
  description: string;
  arguments: ActionArgument[];
  allowed_names: string[];
  group: ActionGroup;
}

export interface FixedPointProperties {
  value: string;
}

export interface FreePointProperties {
  value: string;
}

export interface MidpointProperties {
  point1: string;
  point2: string;
}

export interface IntersectionPointProperties {
  object_name_1: string;
  object_name_2: string;
}

export interface SlidingPointProperties {
  constraining_object_name: string;
  value: string;
}

export interface LineABProperties {
  point1: string;
  point2: string;
}

export interface PpBisectorProperties {
  point1: string;
  point2: string;
}

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface ParameterProperties {
}

export interface InvariantProperties {
  formula: string;
}

export interface LocusProperties {
  point: string;
}

export interface TwoPointDistanceInvariantProperties {
  point1: string;
  point2: string;
}

export interface PointToLineDistanceInvariantProperties {
  point: string;
  line: string;
}

export interface TwoLineAngleInvariantProperties {
  line1: string;
  line2: string;
}

export interface PpToLineProperties {
  point: string;
  line: string;
}

export interface PlToLineProperties {
  point: string;
  line: string;
}

export interface ProjectionProperties {
  point: string;
  line: string;
}

export interface ReflectionProperties {
  point: string;
  line: string;
}

export interface ComputedPointProperties {
  x_expr: string;
  y_expr: string;
  value: string;
}

export interface ScaledVectorPointProperties {
  k: string;
  point1: string;
  point2: string;
  k_value: number;
}

export type ObjectProperties =
  | FixedPointProperties
  | FreePointProperties
  | MidpointProperties
  | IntersectionPointProperties
  | SlidingPointProperties
  | ProjectionProperties
  | ReflectionProperties
  | ComputedPointProperties
  | ScaledVectorPointProperties
  | LineABProperties
  | PpBisectorProperties
  | PpToLineProperties
  | PlToLineProperties
  | ParameterProperties
  | TwoPointDistanceInvariantProperties
  | PointToLineDistanceInvariantProperties
  | TwoLineAngleInvariantProperties
  | InvariantProperties
  | LocusProperties
  | null;

export interface DBObject {
  name: string;
  object_type: ObjectType;
  properties: ObjectProperties;
}

export interface PartialDBObject {
  name: string;
  object_type: ObjectType;
  properties: Partial<ObjectProperties>;
}

export type ArgumentValue = (Vector2d | Shape)[];

export type PlotPointElement = number | { r: number; g: number; b: number };

export interface PlotData {
  points: PlotPointElement[][];
  equation: string;
  formatted_equations: string[];
  time_taken: number;
}

export interface Shape {
  objectType: ObjectType | null;
  state: ShapeState;
  name: string;
  description: string;
  getActionType(): ActionType | null;
  getCoveredPoints(): { x: number; y: number }[];
  getIcon(): React.ReactNode | null;
  clone(): Shape;
  getCanvasShape(canvasProperties?: CanvasProperties, key?: string): React.ReactNode;
  getDefinedPoint(): Vector2d | null;
  closeToPoint(point: Vector2d, delta: number): boolean;
  distanceToPoint(point: Vector2d): number;
}

export interface ShapeCreatorInput {
  objectName: string;
  validatedExpressions: string[];
  expressionValues: number[];
  argumentValues: ArgumentValue[];
  hintedObjectPoint: Vector2d | null;
  locusOrdinal: number | null;
}

export interface ShapeCreator {
  objectType: ObjectType;
  getDBObjectProperties(input: ShapeCreatorInput): Partial<ObjectProperties>;
  getInputForDBObject(dbObject: DBObject, shapes: Shape[]): ShapeCreatorInput;
  createShape(input: ShapeCreatorInput): Shape | null;
}

export interface Line {
  point: Vector2d; // Some point on the line
  n: Vector2d; // Normal vector
}

export interface CanvasProperties {
  topLeft: { x: number; y: number };
  scale: number;
} 

export function argValuesAreEqual(argValue1: ArgumentValue, argValue2: ArgumentValue): boolean {
  return argValue1 != null && argValue2 != null && argValue1.length === argValue2.length &&
      argValue1.every((value, index) => value === argValue2[index]);
}
