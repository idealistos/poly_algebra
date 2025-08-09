import React, { useCallback, useEffect, useMemo, useState } from 'react';
import type { KonvaEventObject } from 'konva/lib/Node';
import { Stage as KonvaStage, Layer, Line, Circle, Text } from 'react-konva';
import { ShapeState, ObjectType } from './enums';
import CanvasPointLayer from './CanvasPointLayer';
import type { Shape, CanvasProperties, DBObject, PlotData } from './types';
import { getShapeCreator } from './utils';
import { IntersectionPointShape } from './shapes/IntersectionPointShape';
import type { Stage } from './Stage';

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
  stage: Stage | null;
  unsetAction: () => void;
  shapes: Shape[];
  setShapes: React.Dispatch<React.SetStateAction<Shape[]>>;
  displayedPlotNames: Set<string>;
  setStatusMessage: (message: string | null) => void;
  plotDataByLocusName: Record<string, PlotData>;
  setPlotDataByLocusName: React.Dispatch<React.SetStateAction<Record<string, PlotData>>>;
  fetchPlotPoints: (locusName: string) => Promise<void>;
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

function SceneCanvas(
  { sceneId,
    stage,
    unsetAction,
    shapes,
    setShapes,
    displayedPlotNames,
    setStatusMessage,
    plotDataByLocusName,
    setPlotDataByLocusName,
    fetchPlotPoints,
  }: SceneCanvasProps) {
  const [canvasProperties, setCanvasProperties] = useState<CanvasProperties | null>(null);
  const [stageUpdateCounter, setStageUpdateCounter] = useState(0);


  // Function to update shape highlighting based on target suggested names
  const updateShapeHighlighting = useCallback((targetSuggestedNames: Set<string>) => {
    const currentSuggestedShapes = shapes.filter(s => s.state === ShapeState.Suggested || s.state === ShapeState.SuggestedSelected);
    const currentSuggestedNames = new Set(currentSuggestedShapes.map(s => s.name));

    // Check if the suggested shapes are different
    if (currentSuggestedNames.size !== targetSuggestedNames.size ||
      !Array.from(currentSuggestedNames).every(name => targetSuggestedNames.has(name))) {
      setShapes(prevShapes =>
        prevShapes.map(shape => {
          const shouldBeSuggested = targetSuggestedNames.has(shape.name);
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
    console.log(`SceneCanvas: sceneId changed to ${sceneId}`);
    if (sceneId !== null) {
      console.log(`SceneCanvas: fetching objects for scene ${sceneId}`);
      fetchDBObjects(sceneId).then(sceneResponse => {
        console.log(`SceneCanvas: received ${sceneResponse.objects.length} objects for scene ${sceneId}`);
        setShapes(sceneResponse.objects
          .reduce((acc, obj) => {
            const shapeCreator = getShapeCreator(obj.object_type);
            const input = shapeCreator.getInputForDBObject(obj, acc);
            const shape = shapeCreator.createShape(input);
            if (shape != null) {
              acc.push(shape);
            }
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
  }, [sceneId, setShapes]);

  // Clean up plotDataByLocusName when Locus objects are removed
  useEffect(() => {
    setPlotDataByLocusName(prev => {
      const currentLocusNames = new Set(
        shapes
          .filter(shape => shape.objectType === ObjectType.Locus)
          .map(shape => shape.name)
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


  useEffect(() => {
    if (stage?.isReady()) {
      const dbObject = stage.getDBObject();
      const shape = stage.getShape();
      if (shape == null) {
        return;
      }

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
        .then(() => {
          shape.state = ShapeState.Default;
          setShapes(prevShapes => [...prevShapes, shape]);
          console.log("Added " + JSON.stringify(shape, null, 2));
          unsetAction();
        })
        .catch(err => {
          console.error('Failed to POST new object or fetch plot points:', err);
          setStatusMessage(`Error: ${err instanceof Error ? err.message : 'Unknown error occurred'}`);
          unsetAction();
        });
    }
  }, [stage, shapes, sceneId, setShapes, setStatusMessage, fetchPlotPoints, unsetAction]);

  // Right-click handler
  const handleContextMenu = useCallback((e: KonvaEventObject<PointerEvent>) => {
    e.evt.preventDefault();
    if (stage) {
      unsetAction();
    }
  }, [stage, unsetAction]);


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

  if (canvasProperties == null) {
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
    const targetStage = e.target.getStage();
    if (!targetStage) return;
    const pointer = targetStage.getPointerPosition();
    if (!pointer) return;

    if (stage != null && !stage.isCurrentArgumentAnExpression()) {
      const physicalToLogical = (px: number, py: number) => {
        const { topLeft, scale } = canvasProperties;
        const x = px * scale + topLeft.x;
        const y = topLeft.y - py * scale;
        return { x, y };
      };

      // Convert pointer to logical coordinates
      const logicalCoords = physicalToLogical(pointer.x, pointer.y);
      const argValueResult = stage.getArgumentValueForCoordinates(logicalCoords);
      let updated = false;

      // Update shape highlighting based on shapesToHighlight
      if (argValueResult != null) {
        const { argValue, shapesToHighlight } = argValueResult;
        updated = stage.setStagedArgument(argValue);
        const targetSuggestedNames = new Set(shapesToHighlight);
        updateShapeHighlighting(targetSuggestedNames);
      } else {
        // Clear all suggested states if no argument value
        clearSuggestedStates();
        updated = stage.setHintedPoint(logicalCoords);
      }
      if (updated) {
        setStageUpdateCounter(prev => prev + 1);
      }
    } else {
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
    if (e.evt.button === 0 && stage != null && stage.canConfirmStagedArgument()) { // Left mouse button
      stage.confirmStagedArgument();
      setStageUpdateCounter(prev => prev + 1);
      if (!stage.isReady()) {
        return;
      }
      console.log("Should create shape in useEffect()");
    }

    if (!canvasProperties) return;
    const targetStage = e.target.getStage();
    if (!targetStage) return;
    const pointer = targetStage.getPointerPosition();
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

  console.log("Before render", stageUpdateCounter);

  return (
    <KonvaStage
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
        {shapes.map(shape => shape.getCanvasShape(canvasProperties!, shape.name))}
        {stage != null && stage.getShape()?.getCanvasShape(canvasProperties!)}
      </Layer>
      <CanvasPointLayer
        plotDataByLocusName={plotDataByLocusName}
        displayedPlotNames={displayedPlotNames}
        shapes={shapes}
      />
    </KonvaStage>
  );
}

export default SceneCanvas;

/*
"Stage" object:

An object encapsulating all what is needed to display the "hinted" and the "staged" objects,
plus to guide SceneCanvas regarding allowed next actions. Null if the object isn't being constructed.

Immutable (set in the constructor):
- action (the struct from the backend)
- verified expressions (just the strings) and their initial values
- shapes
- staged shape name (computed from "shapes")

State:
- current step
- arguments entered so far (names or "x,y"), without names
- hinted object point (if any)

State updates:
- setHintedPoint(point) - when argValue = Null
- setStagedArgument(argValue)
- confirmStagedArgument() - on mouse click

Returns:
- argument hint
- argument types, to use in getArgumentValueForCoordinates()
- state (hinted or staged)
- Shape
- dbObject to send to the backend (in setEffect() of SceneCanvas)
*/