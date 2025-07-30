import React, { useCallback, useEffect, useMemo, useState } from 'react';
import type { KonvaEventObject } from 'konva/lib/Node';
import { Stage, Layer, Line, Circle, Text } from 'react-konva';
import { ShapeState, ObjectType, ArgumentType, getPointDBProperties, getOccupiedPoints, MOBILE_POINT_OBJECT_TYPES } from './enums';
import CanvasPointLayer from './CanvasPointLayer';
import type { Shape, CanvasProperties, Action, ObjectProperties, PartialDBObject, DBObject, PlotData, TwoPointDistanceInvariantProperties, PointToLineDistanceInvariantProperties } from './types';
import type { Vector2d } from 'konva/lib/types';
import { checkLineAlreadyChosen, createShapeForDBObject, getDBObjectForExpressions, getDBPropertiesForLine } from './utils';
import { IntersectionPointShape } from './shapes/IntersectionPointShape';
import { LineBasedShape } from './shapes/LineBasedShape';

export interface View {
  center: {
    x: number;
    y: number;
  };
  diagonal: number;
}

export interface SceneResponse {
  objects: DBObject[];
  view: View;
}

async function fetchDBObjects(sceneId: number): Promise<SceneResponse> {
  const response = await fetch(`http://localhost:8080/scenes/${sceneId}`);
  return response.json();
}

function closeToPoint(mouseX: number, mouseY: number, shape: Shape, canvasProperties: CanvasProperties): boolean {
  const { scale } = canvasProperties;
  let delta = 12 * scale;
  if (shape instanceof IntersectionPointShape) {
    delta *= 2;
  }

  // Convert physical coordinates to logical coordinates
  const logicalX = mouseX * scale + canvasProperties.topLeft.x;
  const logicalY = canvasProperties.topLeft.y - mouseY * scale;

  return shape.closeToPoint({ x: logicalX, y: logicalY }, delta);
}

interface SceneCanvasProps {
  sceneId: number | null;
  currentAction: Action | null;
  currentActionStep: number;
  setCurrentActionStep: React.Dispatch<React.SetStateAction<number>>;
  stagedShapeName: string | null;
  setStagedShapeName: (name: string | null) => void;
  validatedExpressions: string[];
  unsetAction: () => void;
  shapes: Shape[];
  setShapes: React.Dispatch<React.SetStateAction<Shape[]>>;
  displayedPlotNames: Set<string>;
  setStatusMessage: (message: string | null) => void;
  plotDataByLocusName: Record<string, PlotData>;
  setPlotDataByLocusName: React.Dispatch<React.SetStateAction<Record<string, PlotData>>>;
  fetchPlotPoints: (locusName: string) => Promise<void>;
  insertComputableFields: (sceneId: number, dbObject: PartialDBObject) => Promise<PartialDBObject>;
}

function toCanvasProperties(view: View): CanvasProperties {
  const w = window.innerWidth;
  const h = window.innerHeight;
  const scale = view.diagonal / Math.sqrt(w * w + h * h);
  const topLeft = {
    x: view.center.x - scale * w / 2,
    y: view.center.y + scale * h / 2
  };

  return {
    topLeft,
    scale,
  };
}

function getClosestDefinedPoint(
  objectTypes: ObjectType[],
  shapes: Shape[],
  logicalPoint: Vector2d,
  isOccupied: (point: Vector2d) => boolean
): { shape: Shape | null; minDist: number } {
  let minDist = Infinity;
  let closest: Shape | null = null;
  for (const shape of shapes) {
    if (objectTypes.includes(shape.dbObject.object_type)) {
      const definedPoint = shape.getDefinedPoint();
      if (definedPoint) {
        const dist = Math.sqrt(
          Math.pow(logicalPoint.x - definedPoint.x, 2) + Math.pow(logicalPoint.y - definedPoint.y, 2)
        );
        if (dist < minDist && !isOccupied(definedPoint)) {
          minDist = dist;
          closest = shape;
        }
      }
    }
  }
  return { shape: closest, minDist };
}

function getMatchingObjectTypes(argType: ArgumentType): ObjectType[] {
  switch (argType) {
    case ArgumentType.MobilePoint:
      return MOBILE_POINT_OBJECT_TYPES;
    case ArgumentType.AnyDefinedPoint:
    case ArgumentType.AnyDefinedOrGridPoint:
      return [...MOBILE_POINT_OBJECT_TYPES, ObjectType.FixedPoint];
    case ArgumentType.IntersectionPoint:
      return [ObjectType.LineAB, ObjectType.PpBisector];
    case ArgumentType.SlidingPoint:
      return [...MOBILE_POINT_OBJECT_TYPES, ObjectType.FixedPoint];
    case ArgumentType.Line:
      return [ObjectType.LineAB, ObjectType.PpBisector];
    case ArgumentType.GridPoint:
      return [];
    default:
      {
        const exhaustiveCheck: never = argType;
        throw new Error(`Unhandled argument type: ${exhaustiveCheck}`);
      }
  }
}

function getTwoClosestLines(
  shapes: Shape[],
  logicalPoint: Vector2d
): { shape: LineBasedShape; distance: number }[] | null {
  const lineShapes = shapes.filter(shape => shape instanceof LineBasedShape) as LineBasedShape[];

  if (lineShapes.length < 2) {
    return null;
  }

  // Calculate distances to all lines using the distanceToPoint method
  const linesWithDistances = lineShapes.map(line => ({
    shape: line,
    distance: line.distanceToPoint(logicalPoint)
  }));

  // Sort by distance and return the two closest
  linesWithDistances.sort((a, b) => a.distance - b.distance);

  return linesWithDistances.slice(0, 2);
}

function getClosestLine(
  shapes: Shape[],
  logicalPoint: Vector2d
): { shape: LineBasedShape; distance: number } | null {
  const lineShapes = shapes.filter(shape => shape instanceof LineBasedShape) as LineBasedShape[];

  if (lineShapes.length === 0) {
    console.log("No lines found");
    return null;
  }

  // Calculate distances to all lines using the distanceToPoint method
  const linesWithDistances = lineShapes.map(line => ({
    shape: line,
    distance: line.distanceToPoint(logicalPoint)
  }));
  console.log(linesWithDistances);

  // Sort by distance and return the closest
  linesWithDistances.sort((a, b) => a.distance - b.distance);

  return linesWithDistances[0];
}

function getPointArgumentValue(
  logicalPoint: Vector2d,
  argumentTypeIndex: number,
  action: Action,
  actionStep: number,
  shapes: Shape[],
  isOccupied: (point: Vector2d) => boolean,
): { dbProperties: Partial<ObjectProperties>, shapesToHighlight: string[], objectType: ObjectType } | null {
  const objectType = (actionStep == action.arguments.length - 1) ?
    action.object_types[argumentTypeIndex] : action.object_types[0];

  // Helper function to check grid point
  const checkGridPoint = () => {
    const gridX = Math.round(logicalPoint.x);
    const gridY = Math.round(logicalPoint.y);
    const dist = Math.sqrt(
      Math.pow(logicalPoint.x - gridX, 2) + Math.pow(logicalPoint.y - gridY, 2)
    );
    if (dist < 0.15 && !isOccupied({ x: gridX, y: gridY })) {
      return {
        dbProperties: getPointDBProperties(objectType, { x: gridX, y: gridY }, actionStep),
        shapesToHighlight: [],
        objectType,
      };
    }
    return null;
  };

  // Helper function to check defined point
  const checkDefinedPoint = (objectTypes: ObjectType[]) => {
    const { shape: closest, minDist } = getClosestDefinedPoint(objectTypes, shapes, logicalPoint, isOccupied);
    if (closest && minDist < 0.15) {
      const definedPoint = closest.getDefinedPoint()!;
      if (!isOccupied(definedPoint)) {
        return {
          dbProperties: getPointDBProperties(objectType, closest.dbObject.name, actionStep),
          shapesToHighlight: [closest.dbObject.name],
          objectType,
        };
      }
    }
    return null;
  };

  const argType = action.arguments[actionStep].types[argumentTypeIndex];
  switch (argType) {
    case ArgumentType.GridPoint: {
      return checkGridPoint();
    }
    case ArgumentType.MobilePoint:
    case ArgumentType.AnyDefinedPoint: {
      const objectTypes = getMatchingObjectTypes(argType);
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

function getArgumentValueForCoordinates(
  logicalPoint: Vector2d,
  action: Action,
  actionStep: number,
  dbObjectForNextStep: PartialDBObject | null,
  shapes: Shape[],
): { dbProperties: Partial<ObjectProperties>, shapesToHighlight: string[], objectType: ObjectType } | null {
  const argument = action.arguments[actionStep];
  const pointsOccupiedByPartialObject = dbObjectForNextStep ? getOccupiedPoints(dbObjectForNextStep) : [];
  const isOccupied = (point: Vector2d) => shapes.some(
    s => {
      if (argument?.exclusive_object_types?.includes(s.dbObject.object_type)) {
        return s.points.length > 0 &&
          s.points[0].x === point.x &&
          s.points[0].y === point.y;
      }
      return false;
    }
  ) || pointsOccupiedByPartialObject.some(p => p.x === point.x && p.y === point.y);
  for (const [index, argType] of argument?.types?.entries() ?? []) {
    // Handle point-related argument types
    if (argType === ArgumentType.GridPoint ||
      argType === ArgumentType.MobilePoint ||
      argType === ArgumentType.AnyDefinedPoint ||
      argType === ArgumentType.AnyDefinedOrGridPoint) {
      const result = getPointArgumentValue(logicalPoint, index, action, actionStep, shapes, isOccupied);
      if (result) {
        result.objectType = action.object_types[index];
        return result;
      }
      continue;
    }
    switch (argType) {
      case ArgumentType.IntersectionPoint: {
        const twoClosestLines = getTwoClosestLines(shapes, logicalPoint);
        console.log('twoClosestLines', twoClosestLines);
        if (!twoClosestLines) {
          break;
        }
        const [line1, line2] = twoClosestLines;
        if (line1.distance > 0.15 || line2.distance > 0.15) {
          break;
        }
        const intersectionPoint = line1.shape.intersect(line2.shape);
        if (!intersectionPoint || isOccupied(intersectionPoint)) {
          break;
        }
        const distToIntersection = Math.sqrt(
          Math.pow(logicalPoint.x - intersectionPoint.x, 2) +
          Math.pow(logicalPoint.y - intersectionPoint.y, 2)
        );
        if (distToIntersection < 0.25) {
          return {
            dbProperties: {
              object_name_1: line1.shape.dbObject.name,
              object_name_2: line2.shape.dbObject.name,
            },
            shapesToHighlight: [line1.shape.dbObject.name, line2.shape.dbObject.name],
            objectType: ObjectType.IntersectionPoint,
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
          return null;
        }
        const closestLine = getClosestLine(shapes, logicalPoint);
        if (!closestLine || closestLine.distance > 0.15) {
          return null;
        }
        return {
          dbProperties: {
            constraining_object_name: closestLine.shape.dbObject.name,
            value: `${gridX},${gridY}`,
          },
          shapesToHighlight: [closestLine.shape.dbObject.name],
          objectType: ObjectType.SlidingPoint,
        }
      }
      case ArgumentType.Line: {
        const closestLine = getClosestLine(shapes, logicalPoint);
        if (!closestLine || closestLine.distance > 0.15) {
          return null;
        }
        if (actionStep === 1 && dbObjectForNextStep && checkLineAlreadyChosen(dbObjectForNextStep, closestLine.shape)) {
          return null;
        }
        return {
          dbProperties: getDBPropertiesForLine(closestLine.shape, action.object_types[index], actionStep),
          shapesToHighlight: [closestLine.shape.dbObject.name],
          objectType: action.object_types[index],
        };
      }
      default: {
        const exhaustiveCheck: never = argType;
        throw new Error(`Unhandled argument type: ${exhaustiveCheck}`);
      }
    }
  }
  return null;
}

function convertDBObject(dbObject: PartialDBObject, newObjectType: ObjectType): PartialDBObject {
  if (newObjectType === ObjectType.PointToLineDistanceInvariant && dbObject.object_type === ObjectType.TwoPointDistanceInvariant) {
    (dbObject.properties as Partial<PointToLineDistanceInvariantProperties>).point = (dbObject.properties as Partial<TwoPointDistanceInvariantProperties>).point1;
    delete (dbObject.properties as Partial<TwoPointDistanceInvariantProperties>).point1;
  }
  dbObject.object_type = newObjectType;
  return dbObject;
}

function SceneCanvas(
  { sceneId,
    currentAction,
    currentActionStep,
    setCurrentActionStep,
    stagedShapeName,
    setStagedShapeName,
    validatedExpressions,
    unsetAction,
    shapes,
    setShapes,
    displayedPlotNames,
    setStatusMessage,
    plotDataByLocusName,
    setPlotDataByLocusName,
    fetchPlotPoints,
    insertComputableFields,
  }: SceneCanvasProps) {
  const [canvasProperties, setCanvasProperties] = useState<CanvasProperties | null>(null);
  const [stagedObject, setStagedObject] = useState<Shape | null>(null);
  const [objectHint, setObjectHint] = useState<Shape | null>(null);
  const [dbObjectForNextStep, setDBObjectForNextStep] = useState<PartialDBObject | null>(null);
  const [fieldsComputed, setFieldsComputed] = useState(false);

  const unsetActionAndObjects = useCallback(() => {
    unsetAction();
    setStagedObject(null);
    setObjectHint(null);
    setDBObjectForNextStep(null);
    setFieldsComputed(false);
  }, [unsetAction, setStagedObject, setObjectHint, setDBObjectForNextStep]);

  // Function to update shape highlighting based on target suggested names
  const updateShapeHighlighting = useCallback((targetSuggestedNames: Set<string>) => {
    const currentSuggestedShapes = shapes.filter(s => s.state === ShapeState.Suggested || s.state === ShapeState.SuggestedSelected);
    const currentSuggestedNames = new Set(currentSuggestedShapes.map(s => s.dbObject.name));

    // Check if the suggested shapes are different
    if (currentSuggestedNames.size !== targetSuggestedNames.size ||
      !Array.from(currentSuggestedNames).every(name => targetSuggestedNames.has(name))) {
      setShapes(prevShapes =>
        prevShapes.map(shape => {
          const shouldBeSuggested = targetSuggestedNames.has(shape.dbObject.name);
          if (shape.state === ShapeState.Default && shouldBeSuggested) {
            shape.state = ShapeState.Suggested;
            return shape.clone();
          } else if (shape.state === ShapeState.Suggested && !shouldBeSuggested) {
            shape.state = ShapeState.Default;
            return shape.clone();
          } else if (shape.state === ShapeState.Selected && shouldBeSuggested) {
            shape.state = ShapeState.SuggestedSelected;
            return shape.clone();
          } else if (shape.state === ShapeState.SuggestedSelected && !shouldBeSuggested) {
            shape.state = ShapeState.Selected;
            return shape.clone();
          }
          return shape;
        })
      );
    }
  }, [shapes, setShapes]);

  // Function to clear all suggested states
  const clearSuggestedStates = useCallback(() => {
    const hasSuggestedShapes = shapes.some(s => s.state === ShapeState.Suggested || s.state === ShapeState.SuggestedSelected);
    if (hasSuggestedShapes) {
      setShapes(prevShapes =>
        prevShapes.map(shape => {
          if (shape.state === ShapeState.Suggested) {
            shape.state = ShapeState.Default;
            return shape.clone();
          } else if (shape.state === ShapeState.SuggestedSelected) {
            shape.state = ShapeState.Selected;
            return shape.clone();
          }
          return shape;
        })
      );
    }
  }, [shapes, setShapes]);

  useEffect(() => {
    const computeFields = async () => {
      const dbObjectWithComputedFields = await insertComputableFields(sceneId!, dbObjectForNextStep!);
      setDBObjectForNextStep(dbObjectWithComputedFields);
      setFieldsComputed(true);
    };
    if (sceneId &&
      dbObjectForNextStep &&
      currentAction &&
      currentActionStep > 0 &&
      currentAction.arguments[currentActionStep - 1].types.length == 0 &&
      !fieldsComputed) {
      computeFields();
    }
  }, [currentAction, currentActionStep, sceneId, fieldsComputed, insertComputableFields, dbObjectForNextStep]);

  useEffect(() => {
    console.log(`SceneCanvas: sceneId changed to ${sceneId}`);
    if (sceneId !== null) {
      console.log(`SceneCanvas: fetching objects for scene ${sceneId}`);
      fetchDBObjects(sceneId).then(sceneResponse => {
        console.log(`SceneCanvas: received ${sceneResponse.objects.length} objects for scene ${sceneId}`);
        setShapes(sceneResponse.objects
          .reduce((acc, obj) => {
            const shape = createShapeForDBObject(obj, acc, -1);
            acc.push(shape);
            return acc;
          }, [] as Shape[]));
        setCanvasProperties(toCanvasProperties(sceneResponse.view));
      }).catch(err => {
        console.error(`SceneCanvas: failed to fetch objects for scene ${sceneId}:`, err);
      });
    } else {
      console.log(`SceneCanvas: sceneId is null, clearing shapes`);
      setShapes([]);
    }
  }, [sceneId, setShapes, currentActionStep]);

  // Clean up plotDataByLocusName when Locus objects are removed
  useEffect(() => {
    setPlotDataByLocusName(prev => {
      const currentLocusNames = new Set(
        shapes
          .filter(shape => shape.dbObject.object_type === ObjectType.Locus)
          .map(shape => shape.dbObject.name)
      );

      const newPlotData = { ...prev };
      let hasChanges = false;

      for (const locusName in newPlotData) {
        if (!currentLocusNames.has(locusName)) {
          delete newPlotData[locusName];
          hasChanges = true;
        }
      }

      return hasChanges ? newPlotData : prev;
    });
  }, [shapes, setPlotDataByLocusName]);

  // Compute stagedShapeName if needed
  useEffect(() => {
    if (currentAction && !stagedShapeName) {
      const existingNames = new Set(shapes.map(s => s.dbObject.name));
      let found = null;
      // Try allowed_names directly
      for (const name of currentAction.allowed_names) {
        if (!existingNames.has(name)) {
          found = name;
          break;
        }
      }
      // Try allowed_names with suffixes
      if (!found) {
        for (let suffix = 1; suffix < 100; ++suffix) {
          for (const name of currentAction.allowed_names) {
            const candidate = name + suffix;
            if (!existingNames.has(candidate)) {
              found = candidate;
              break;
            }
          }
          if (found) break;
        }
      }
      if (found) {
        // Check if the arguments list is empty (e.g., ObjectType.Parameter)
        if (currentAction.arguments.length === 0) {
          // Create the shape immediately
          const dbObject = {
            name: found,
            object_type: currentAction.object_types[0]!,
            properties: null,
          };
          const newShape = createShapeForDBObject(dbObject, shapes, currentActionStep);
          newShape.state = ShapeState.Default;
          setShapes(prevShapes => [...prevShapes, newShape]);
          console.log("Added " + JSON.stringify(newShape, null, 2));

          // POST to backend
          fetch(`http://localhost:8080/scenes/${sceneId}/objects`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(dbObject),
          })
            .then(res => {
              if (!res.ok) {
                return res.text().then(text => { throw new Error(text || res.statusText); });
              }
            })
            .then(() => {
              // If the object is a Locus, fetch its plot points
              if (dbObject.object_type === ObjectType.Locus) {
                return fetchPlotPoints(dbObject.name);
              }
            })
            .catch(err => {
              console.error('Failed to POST new object or fetch plot points:', err);
              setStatusMessage(`Error: ${err instanceof Error ? err.message : 'Unknown error occurred'}`);
            });

          unsetAction();
        } else {
          setStagedShapeName(found);
        }
      }
    }
    if (!currentAction && stagedShapeName) {
      setStagedShapeName(null);
    }
  }, [currentAction, stagedShapeName, shapes, setStagedShapeName, sceneId, setShapes, setStatusMessage, fetchPlotPoints, unsetAction, currentActionStep]);


  useEffect(() => {
    if (validatedExpressions.length > 0 && currentActionStep > 0
      && dbObjectForNextStep == null
      && currentAction
      && currentAction.arguments[currentActionStep].types.length > 0
      && stagedShapeName
    ) {
      setDBObjectForNextStep(getDBObjectForExpressions(stagedShapeName, validatedExpressions, currentAction));
    }
  }, [validatedExpressions, currentAction, currentActionStep, dbObjectForNextStep, stagedShapeName]);

  // Right-click handler
  const handleContextMenu = useCallback((e: KonvaEventObject<PointerEvent>) => {
    e.evt.preventDefault();
    if (currentAction) {
      unsetActionAndObjects();
    }
  }, [currentAction, unsetActionAndObjects]);


  // Compute covered points
  const coveredPoints = useMemo(() => {
    const coveredPoints = new Set<string>();
    shapes.forEach(shape => {
      shape.getCoveredPoints().forEach(point => {
        coveredPoints.add(`${point.x},${point.y}`);
      });
    });
    return coveredPoints;
  }, [shapes]);

  // Canvas size
  const width = window.innerWidth;
  const height = window.innerHeight;

  // Logical bounds
  const { topLeft, scale } = canvasProperties ?? { topLeft: { x: 0, y: 0 }, scale: 1 };


  // Grid points (integer logical coordinates within visible area)
  const gridPoints = useMemo(() => {
    const gridPoints = [];
    const logicalLeft = topLeft.x;
    const logicalTop = topLeft.y;
    const logicalRight = topLeft.x + width * scale;
    const logicalBottom = topLeft.y - height * scale;
    for (let x = Math.ceil(logicalLeft); x <= Math.floor(logicalRight); x++) {
      for (let y = Math.floor(logicalBottom); y <= Math.ceil(logicalTop); y++) {
        // Skip if this point is covered by a shape
        if (coveredPoints.has(`${x},${y}`)) {
          continue;
        }
        // Convert logical (x, y) to physical (px, py)
        const px = (x - topLeft.x) / scale;
        const py = (topLeft.y - y) / scale;
        gridPoints.push({ px, py, x, y });
      }
    }
    return gridPoints;
  }, [coveredPoints, width, height, scale, topLeft]);

  if (!canvasProperties) {
    return null;
  }

  // Axes: X (y=0), Y (x=0)
  // X axis: from left to right at y=0
  const xAxisY = (topLeft.y - 0) / scale;
  // Y axis: from top to bottom at x=0
  const yAxisX = (0 - topLeft.x) / scale;

  // Origin (0,0) in physical coordinates
  const originPx = (0 - topLeft.x) / scale;
  const originPy = (topLeft.y - 0) / scale;

  // Grid point radius
  const gridRadius = Math.max(0.05 / scale, 1);

  // Mouse move handler
  const handleMouseMove = (e: KonvaEventObject<MouseEvent>) => {
    if (!canvasProperties) return;
    const stage = e.target.getStage();
    if (!stage) return;
    const pointer = stage.getPointerPosition();
    if (!pointer) return;

    if (currentAction != null
      && currentAction.arguments[currentActionStep] != null
      && currentAction.arguments[currentActionStep].types.length > 0) {
      const physicalToLogical = (px: number, py: number) => {
        const { topLeft, scale } = canvasProperties;
        const x = px * scale + topLeft.x;
        const y = topLeft.y - py * scale;
        return { x, y };
      };

      // Convert pointer to logical coordinates
      const logicalCoords = physicalToLogical(pointer.x, pointer.y);
      const argValue = getArgumentValueForCoordinates(logicalCoords, currentAction, currentActionStep, dbObjectForNextStep, shapes);
      console.log("argValue", argValue);

      // Update shape highlighting based on shapesToHighlight
      if (argValue) {
        const targetSuggestedNames = new Set(argValue.shapesToHighlight);
        updateShapeHighlighting(targetSuggestedNames);

        // Handle staged object
        if (objectHint) setObjectHint(null);
        setStagedObject(prevStaged => {
          const objectType = argValue.objectType;
          const name = stagedShapeName || 'staged';
          if (prevStaged && !dbObjectForNextStep) {
            prevStaged.dbObject.object_type = objectType;
            prevStaged.dbObject.properties = { ...prevStaged.dbObject.properties, ...argValue.dbProperties } as ObjectProperties;
            console.log('prevStaged', prevStaged.dbObject);
            return prevStaged.clone();
          }
          let dbObject;
          if (dbObjectForNextStep) {
            console.log('dbObjectForNextStep', dbObjectForNextStep);
            dbObject = { ...dbObjectForNextStep };
            dbObject.properties = { ...dbObject.properties, ...argValue.dbProperties } as ObjectProperties;
            if (objectType !== dbObject.object_type) {
              dbObject = convertDBObject(dbObject, objectType);
            }
          } else {
            dbObject = { name, object_type: objectType, properties: argValue.dbProperties as ObjectProperties };
          }
          console.log('staged', dbObject);
          const stagedObject = createShapeForDBObject(dbObject, shapes, currentActionStep);
          stagedObject.state = ShapeState.BeingAdded;
          return stagedObject;
        });
      } else {
        // Clear all suggested states if no argument value
        clearSuggestedStates();

        // Handle object hint
        if (stagedObject) setStagedObject(null);
        setObjectHint(prevHint => {
          const objectType = currentAction!.object_types[0];
          const name = stagedShapeName || 'hint';
          if (prevHint) {
            prevHint.updatePoints(currentActionStep, logicalCoords);
            return prevHint.clone();
          }
          console.log('Using dbObjectForNextStep', dbObjectForNextStep);
          const dbObject = dbObjectForNextStep ?? { name, object_type: objectType, properties: null };
          const objectHint = createShapeForDBObject(dbObject, shapes, currentActionStep);
          console.log('dbObject', dbObject);
          console.log('objectHint', objectHint);
          objectHint.updatePoints(currentActionStep, logicalCoords);
          objectHint.state = ShapeState.Hinted;
          return objectHint;
        });
      }
    } else {
      if (stagedObject) setStagedObject(null);
      if (objectHint) setObjectHint(null);
      setShapes(prevShapes =>
        prevShapes.map(shape => {
          const isClose = closeToPoint(pointer.x, pointer.y, shape, canvasProperties);
          if (shape.state === ShapeState.Default || shape.state === ShapeState.Suggested) {
            shape.state = isClose ? ShapeState.Suggested : ShapeState.Default;
            return shape.clone();
          } else if (shape.state === ShapeState.Selected || shape.state === ShapeState.SuggestedSelected) {
            shape.state = isClose ? ShapeState.SuggestedSelected : ShapeState.Selected;
            return shape.clone();
          }
          return shape;
        })
      );
    }
  };

  // Mouse click handler
  const handleMouseDown = async (e: KonvaEventObject<MouseEvent>) => {
    if (e.evt.button === 0 && stagedObject) { // Left mouse button
      if (currentAction && currentActionStep < (currentAction.arguments.length ?? 0) - 1) {
        console.log('New dbObjectForNextStep', stagedObject.getDBObjectForNextStep());
        setDBObjectForNextStep(stagedObject.getDBObjectForNextStep());
        setStatusMessage(currentAction.arguments[currentActionStep + 1].hint);
        setCurrentActionStep(prev => prev + 1);
        return;
      }

      stagedObject.state = ShapeState.Default;
      fetch(`http://localhost:8080/scenes/${sceneId}/objects`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(stagedObject.dbObject),
      })
        .then(res => {
          if (!res.ok) {
            return res.text().then(text => { throw new Error(text || res.statusText); });
          }
        })
        .then(() => {
          setShapes(prevShapes => [...prevShapes, stagedObject.clone()]);
          // If the object is a Locus, fetch its plot points
          let result = null;
          if (stagedObject.dbObject.object_type === ObjectType.Locus) {
            result = fetchPlotPoints(stagedObject.dbObject.name);
          }
          unsetActionAndObjects();
          return result;
        })
        .catch(err => {
          console.error('Failed to POST new object or fetch plot points:', err);
          setStatusMessage(`Error: ${err instanceof Error ? err.message : 'Unknown error occurred'}`);
        });
    }

    if (!canvasProperties) return;
    const stage = e.target.getStage();
    if (!stage) return;
    const pointer = stage.getPointerPosition();
    if (!pointer) return;
    const mouseX = pointer.x;
    const mouseY = pointer.y;

    setShapes(prevShapes => {
      // Find the first shape under the mouse
      const selectedIndex = prevShapes.findIndex(shape =>
        closeToPoint(mouseX, mouseY, shape, canvasProperties)
      );
      return prevShapes.map((shape, idx) => {
        shape.state = idx === selectedIndex ? ShapeState.Selected : ShapeState.Default;
        return shape.clone();
      });
    });
  };

  return (
    <Stage
      width={width}
      height={height}
      style={{ position: 'absolute', top: 0, left: 0, zIndex: 0 }}
      onMouseMove={handleMouseMove}
      onMouseDown={handleMouseDown}
      onContextMenu={handleContextMenu}
    >
      <Layer>
        {/* Grid points */}
        {gridPoints.map((pt, i) => (
          <Circle
            key={i}
            x={pt.px}
            y={pt.py}
            radius={gridRadius}
            fill="#888"
          />
        ))}
        {/* X axis (dotted line) */}
        {xAxisY >= 0 && xAxisY <= height && (
          <Line
            points={[0, xAxisY, width, xAxisY]}
            stroke="#bbb"
            strokeWidth={1}
            dash={[6, 6]}
          />
        )}
        {/* Y axis (dotted line) */}
        {yAxisX >= 0 && yAxisX <= width && (
          <Line
            points={[yAxisX, 0, yAxisX, height]}
            stroke="#bbb"
            strokeWidth={1}
            dash={[6, 6]}
          />
        )}
        {/* Origin label */}
        {originPx >= 0 && originPx <= width && originPy >= 0 && originPy <= height && (
          <Text
            x={originPx + 6}
            y={originPy + 6}
            fontSize={12}
            fill="#333"
            text="0,0"
          />
        )}
      </Layer>
      <Layer>
        {shapes.map(shape =>
          shape.getCanvasShape(canvasProperties!, shape.dbObject.name)
        )}
        {stagedObject && stagedObject.getCanvasShape(canvasProperties!)}
        {objectHint && objectHint.getCanvasShape(canvasProperties!)}
      </Layer>
      <CanvasPointLayer
        plotDataByLocusName={plotDataByLocusName}
        displayedPlotNames={displayedPlotNames}
        shapes={shapes}
      />
    </Stage>
  );
}

export default SceneCanvas; 