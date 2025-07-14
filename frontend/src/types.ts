import { ActionType, ArgumentType, ObjectType, ShapeState } from './enums';
import type { Vector2d } from 'konva/lib/types';

export interface ActionArgument {
  types: ArgumentType[];
  hint: string;
  exclusive_object_types: ObjectType[];
}

export interface Action {
  name: ActionType;
  object_type: ObjectType;
  description: string;
  arguments: ActionArgument[];
  allowed_names: string[];
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

export interface InvariantProperties {
  formula: string;
}

export interface LocusProperties {
  point: string;
}

export type ObjectProperties =
  | FixedPointProperties
  | FreePointProperties
  | MidpointProperties
  | IntersectionPointProperties
  | SlidingPointProperties
  | LineABProperties
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

export type PlotPointElement = number | { r: number; g: number; b: number };

export interface Shape {
  dbObject: PartialDBObject;
  state: ShapeState;
  points: Vector2d[];
  getActionType(): ActionType | null;
  getCoveredPoints(): { x: number; y: number }[];
  getIcon(): React.ReactNode | null;
  getDescription(): string;
  clone(): Shape;
  getCanvasShape(canvasProperties?: CanvasProperties, key?: string): React.ReactNode;
  getDBObjectForNextStep(): PartialDBObject | null;
  getDefinedPoint(): Vector2d | null;
  closeToPoint(point: Vector2d, delta: number): boolean;
  distanceToPoint(point: Vector2d): number;
}

export interface CanvasProperties {
  topLeft: { x: number; y: number };
  scale: number;
} 