import React, { useCallback, useEffect, useState } from 'react';
import SceneCanvas from './SceneCanvas';
import { ActionRibbon } from './ActionRibbon';
import { ActionType, ObjectType, ShapeState } from './enums';
import type { Action, Shape, PlotData } from './types';
import './App.css';
import { createShapeForDBObject, PLOT_COLORS } from './utils';
import { SceneManagementModal } from './SceneManagementModal';
import { ConfirmationModal } from './ConfirmationModal';
import Legend from './Legend';
import type { LocusShape } from './shapes/LocusShape';

function InvariantModal({
  mousePos,
  formula,
  setFormula,
  onSubmit,
  onCancel,
  visible
}: {
  mousePos: { x: number, y: number },
  formula: string,
  setFormula: (s: string) => void,
  onSubmit: () => void,
  onCancel: () => void,
  visible: boolean
}) {
  if (!visible) return null;
  return (
    <div
      className="modal-overlay"
      style={{
        left: mousePos.x + 24,
        top: mousePos.y - 16,
      }}
    >
      <div className="modal-content">
        <div className="modal-title">Add Invariant</div>
        <input
          type="text"
          className="modal-input"
          value={formula}
          onChange={e => setFormula(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter') onSubmit(); }}
          placeholder="Enter the formula of the invariant"
          autoFocus
        />
        <div className="modal-buttons">
          <button className="modal-button modal-button-cancel" onClick={onCancel}>Cancel</button>
          <button className="modal-button modal-button-ok" onClick={onSubmit} disabled={!formula.trim()}>OK</button>
        </div>
      </div>
    </div>
  );
}

async function deleteShape(
  shape: Shape,
  setShapes: React.Dispatch<React.SetStateAction<Shape[]>>,
  sceneId: number,
  setStatusMessage: (message: string | null) => void,
  setDisplayedPlotNames?: React.Dispatch<React.SetStateAction<Set<string>>>,
  setConfirmationModal?: React.Dispatch<React.SetStateAction<{
    isOpen: boolean;
    title: string;
    message: string;
    dependents: string[];
    onConfirm: () => void;
  }>>
) {
  try {
    // First, check for dependents
    const dependentsResponse = await fetch(`http://localhost:8080/scenes/${sceneId}/${shape.dbObject.name}/dependents`);

    if (!dependentsResponse.ok) {
      const text = await dependentsResponse.text();
      throw new Error(text || dependentsResponse.statusText);
    }

    const dependents: string[] = await dependentsResponse.json();

    // Filter out the object itself from dependents for the confirmation message
    const otherDependents = dependents.filter(name => name !== shape.dbObject.name);

    // If there are other dependents, show confirmation modal
    if (otherDependents.length > 0 && setConfirmationModal) {
      setConfirmationModal({
        isOpen: true,
        title: 'Delete Object',
        message: `Are you sure you want to delete "${shape.dbObject.name}"?`,
        dependents: otherDependents,
        onConfirm: async () => {
          // Close the modal
          setConfirmationModal(prev => ({ ...prev, isOpen: false }));

          // Proceed with deletion
          await performDeletion(shape, sceneId, setShapes, setStatusMessage, setDisplayedPlotNames);
        }
      });
      return; // Exit early, deletion will be handled by onConfirm
    }

    // No dependents, proceed with deletion directly
    await performDeletion(shape, sceneId, setShapes, setStatusMessage, setDisplayedPlotNames);
  } catch (err) {
    console.error('Failed to delete shape:', err);
    setStatusMessage(`Error: ${err instanceof Error ? err.message : 'Unknown error occurred'}`);
  }
}

async function performDeletion(
  shape: Shape,
  sceneId: number,
  setShapes: React.Dispatch<React.SetStateAction<Shape[]>>,
  setStatusMessage: (message: string | null) => void,
  setDisplayedPlotNames?: React.Dispatch<React.SetStateAction<Set<string>>>
) {
  const response = await fetch(`http://localhost:8080/scenes/${sceneId}/${shape.dbObject.name}`, {
    method: 'DELETE',
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || response.statusText);
  }

  // Parse the response as a list of names to delete
  const deletedNames: string[] = await response.json();

  // Remove all shapes whose names are in the deletedNames list, plus the original shape
  setShapes(prevShapes => {
    const filteredShapes = prevShapes.filter(
      s => s.dbObject.name !== shape.dbObject.name && !deletedNames.includes(s.dbObject.name)
    );

    // Update locusOrdinal for remaining Locus shapes
    const remainingLocusShapes = filteredShapes.filter(s => s.dbObject.object_type === 'Locus');
    remainingLocusShapes.forEach((locusShape, index) => {
      if ('locusOrdinal' in locusShape) {
        (locusShape as LocusShape).locusOrdinal = index;
      }
    });

    return filteredShapes;
  });

  // Clean up plot-related state for Locus objects
  if (shape.dbObject.object_type === 'Locus') {
    // Remove from displayedPlotNames
    setDisplayedPlotNames?.(prevNames => {
      const newNames = new Set(prevNames);
      newNames.delete(shape.dbObject.name);
      return newNames;
    });
  }

  // Also clean up any Locus objects that were deleted as dependencies
  // Remove all deleted names from displayedPlotNames (they might be Locus objects)
  if (deletedNames.length > 0) {
    setDisplayedPlotNames?.(prevNames => {
      const newNames = new Set(prevNames);
      deletedNames.forEach(name => newNames.delete(name));
      return newNames;
    });
  }
}

interface SceneInfo {
  id: number;
  name: string;
  created_at: string;
}

function App() {
  const [scenes, setScenes] = useState<SceneInfo[]>([]);
  const [selectedSceneId, setSelectedSceneId] = useState<number | null>(null);
  const [isCreatingScene, setIsCreatingScene] = useState(false);
  const [newSceneName, setNewSceneName] = useState('');
  const [currentAction, setCurrentAction] = useState<Action | null>(null);
  const [currentActionStep, setCurrentActionStep] = useState<number>(0);
  const [stagedShapeName, setStagedShapeName] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [invariantFormula, setInvariantFormula] = useState('');
  const [mousePos, setMousePos] = useState<{ x: number, y: number }>({ x: 200, y: 200 });
  const [shapes, setShapes] = useState<Shape[]>([]);
  const [displayedPlotNames, setDisplayedPlotNames] = useState<Set<string>>(new Set());
  const [plotDataByLocusName, setPlotDataByLocusName] = useState<Record<string, PlotData>>({});
  const [isSceneManagementModalOpen, setIsSceneManagementModalOpen] = useState(false);
  const [confirmationModal, setConfirmationModal] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    dependents: string[];
    onConfirm: () => void;
  }>({
    isOpen: false,
    title: '',
    message: '',
    dependents: [],
    onConfirm: () => { },
  });

  const unsetAction = useCallback(() => {
    setCurrentAction(null);
    setStagedShapeName(null);
    setCurrentActionStep(0);
    setStatusMessage(null);
    setInvariantFormula('');
  }, [setCurrentAction, setStagedShapeName, setCurrentActionStep, setStatusMessage, setInvariantFormula]);

  useEffect(() => {
    fetch('http://localhost:8080/scenes')
      .then(res => res.json())
      .then((sceneInfos: SceneInfo[]) => {
        setScenes(sceneInfos);
        // Only auto-select if there are scenes and no scene is currently selected
        if (sceneInfos.length > 0 && selectedSceneId === null) {
          setSelectedSceneId(sceneInfos[sceneInfos.length - 1].id);
        }
      })
      .catch(() => setScenes([]));
  }, [selectedSceneId]); // Only run once on mount

  const refreshScenes = async () => {
    try {
      const response = await fetch('http://localhost:8080/scenes');
      const sceneInfos: SceneInfo[] = await response.json();
      setScenes(sceneInfos);
    } catch (err) {
      console.error('Failed to refresh scenes:', err);
      setScenes([]);
    }
  };

  const handleCreateScene = async () => {
    if (!newSceneName.trim()) return;

    try {
      const response = await fetch('http://localhost:8080/scenes', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: newSceneName.trim() }),
      });

      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || response.statusText);
      }

      const newScene: SceneInfo = await response.json();
      // Refresh the entire scene list to ensure consistency
      await refreshScenes();
      setSelectedSceneId(newScene.id);
      setIsCreatingScene(false);
      setNewSceneName('');
      setStatusMessage(`Created new scene: ${newScene.name}`);
    } catch (err) {
      console.error('Failed to create scene:', err);
      setStatusMessage(`Error: ${err instanceof Error ? err.message : 'Unknown error occurred'}`);
    }
  };

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (currentAction?.name !== ActionType.Invariant) {
        setMousePos({ x: e.clientX, y: e.clientY });
      }
    };
    window.addEventListener('mousemove', handler);
    return () => window.removeEventListener('mousemove', handler);
  }, [currentAction]);

  useEffect(() => {
    if (currentAction?.name === ActionType.Invariant) {
      setInvariantFormula('');
    }
  }, [currentAction]);

  // Reset state when scene changes
  useEffect(() => {
    setStagedShapeName(null);
    setStatusMessage(null);
    setDisplayedPlotNames(new Set());
    setPlotDataByLocusName({});
    setShapes([]);
  }, [selectedSceneId]);

  const handleActionClick = useCallback((action: Action) => {
    unsetAction();
    setCurrentAction(action);
    setStatusMessage(action.arguments[0]?.hint ?? null);
  }, [unsetAction, setCurrentAction, setStatusMessage]);

  const selectShape = useCallback((shape: Shape) => {
    const shapeCopy = shape.clone();
    shapeCopy.state = ShapeState.Selected;
    setShapes(prevShapes => prevShapes.map(
      s => {
        if (s.dbObject.name === shape.dbObject.name) {
          return shapeCopy;
        } else if (s.state === ShapeState.Selected) {
          s.state = ShapeState.Default;
          return s.clone();
        }
        return s;
      }));
  }, [setShapes]);

  const handleDeleteShape = useCallback((shape: Shape) => {
    if (selectedSceneId !== null) {
      deleteShape(shape, setShapes, selectedSceneId, setStatusMessage, setDisplayedPlotNames, setConfirmationModal);
    }
  }, [selectedSceneId, setShapes, setStatusMessage, setDisplayedPlotNames, setConfirmationModal]);

  // Handle keyboard events for deleting selected shapes
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Backspace') {
        // Check if any text input field is focused
        const activeElement = document.activeElement;
        const isTextInputFocused = activeElement && (
          activeElement.tagName === 'INPUT' ||
          activeElement.tagName === 'TEXTAREA' ||
          (activeElement instanceof HTMLElement && activeElement.contentEditable === 'true')
        );

        // Only handle backspace for shape deletion if no text input is focused
        if (!isTextInputFocused) {
          // Find the currently selected shape
          const selectedShape = shapes.find(shape =>
            shape.state === ShapeState.Selected || shape.state === ShapeState.SuggestedSelected
          );

          if (selectedShape) {
            e.preventDefault(); // Prevent default backspace behavior
            handleDeleteShape(selectedShape);
          }
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [shapes, handleDeleteShape]);

  // Reusable function to fetch plot points for a locus
  const fetchPlotPoints = useCallback(async (locusName: string) => {
    try {
      setStatusMessage("Computing the curve...");
      const response = await fetch(`http://localhost:8080/scenes/${selectedSceneId}/plot/${locusName}?width=${window.innerWidth}&height=${window.innerHeight}`);
      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || response.statusText);
      }
      const plotData: PlotData = await response.json();

      // Store the complete plot data
      setPlotDataByLocusName(prev => ({
        ...prev,
        [locusName]: plotData
      }));
      console.log(`Saved ${plotData.points.length} points for locus ${locusName}`);
      console.log(`Curve equation: ${plotData.equation}`);

      // Add to displayed plot names
      setDisplayedPlotNames(prev => new Set([...prev, locusName]));

      // Update status message with point count and equation
      const equationText = plotData.formatted_equations.length > 0
        ? plotData.formatted_equations.join(' × ')
        : plotData.equation;
      setStatusMessage(`Computed the curve (point count: ${plotData.points.length}, equation: ${equationText})`);
    } catch (err) {
      console.error(`Failed to fetch plot points for locus ${locusName}:`, err);
      setStatusMessage(`Error: Failed to fetch plot points for locus ${locusName}: ${err instanceof Error ? err.message : 'Unknown error occurred'}`);
      throw err;
    }
  }, [selectedSceneId, setPlotDataByLocusName, setDisplayedPlotNames, setStatusMessage]);

  // Helper function to get locus ordinal number
  const getLocusOrdinal = useCallback((locusName: string) => {
    const locusShapes = shapes.filter(shape =>
      shape.dbObject.object_type === ObjectType.Locus
    );
    return locusShapes.findIndex(shape => shape.dbObject.name === locusName) % 10;
  }, [shapes]);

  const handleTogglePlot = useCallback(async (shapeName: string) => {
    const isCurrentlyDisplayed = displayedPlotNames.has(shapeName);

    if (isCurrentlyDisplayed) {
      // Turn off: just remove from displayed names
      setDisplayedPlotNames(prevNames => {
        const newNames = new Set(prevNames);
        newNames.delete(shapeName);
        return newNames;
      });
    } else {
      // Turn on: check if plot data exists, fetch if needed
      if (!plotDataByLocusName[shapeName]) {
        try {
          await fetchPlotPoints(shapeName);
        } catch {
          return; // Don't add to displayed names if fetch failed
        }
      } else {
        // Plot data already exists, just add to displayed names
        setDisplayedPlotNames(prevNames => {
          const newNames = new Set(prevNames);
          newNames.add(shapeName);
          return newNames;
        });
      }
    }
  }, [displayedPlotNames, plotDataByLocusName, fetchPlotPoints]);

  const handleInvariantSubmit = async () => {
    if (!selectedSceneId || !stagedShapeName || !invariantFormula.trim()) return;
    const dbObject = {
      name: stagedShapeName,
      object_type: ObjectType.Invariant,
      properties: { formula: invariantFormula.trim() },
    };
    try {
      const res = await fetch(`http://localhost:8080/scenes/${selectedSceneId}/objects`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(dbObject),
      });
      if (!res.ok) {
        const text = await res.text();
        throw new Error(text || res.statusText);
      }
      unsetAction();
      console.log('1');
      setShapes(prevShapes => [...prevShapes, createShapeForDBObject(dbObject, shapes, -1)]);
    } catch (err) {
      console.error('Failed to POST invariant:', err);
      setStatusMessage(`Error: ${err instanceof Error ? err.message : 'Unknown error occurred'}`);
    }
  };

  return (
    <div className="app-container">
      <SceneCanvas
        sceneId={selectedSceneId}
        currentAction={currentAction}
        currentActionStep={currentActionStep}
        setCurrentActionStep={setCurrentActionStep}
        stagedShapeName={stagedShapeName}
        setStagedShapeName={setStagedShapeName}
        unsetAction={unsetAction}
        shapes={shapes}
        setShapes={setShapes}
        displayedPlotNames={displayedPlotNames}
        setStatusMessage={setStatusMessage}
        plotDataByLocusName={plotDataByLocusName}
        setPlotDataByLocusName={setPlotDataByLocusName}
        fetchPlotPoints={fetchPlotPoints}
      />
      <ActionRibbon onActionClick={handleActionClick} setStatusMessage={setStatusMessage} />
      <div className="top-bar">
        <div className="scene-selector">
          {isCreatingScene ? (
            <div className="scene-create-input">
              <input
                type="text"
                value={newSceneName}
                onChange={e => setNewSceneName(e.target.value)}
                onKeyDown={e => {
                  if (e.key === 'Enter') {
                    handleCreateScene();
                  } else if (e.key === 'Escape') {
                    setIsCreatingScene(false);
                    setNewSceneName('');
                  }
                }}
                placeholder="Enter scene name..."
                autoFocus
              />
              <button onClick={handleCreateScene} disabled={!newSceneName.trim()}>
                Create
              </button>
              <button onClick={() => {
                setIsCreatingScene(false);
                setNewSceneName('');
              }}>
                Cancel
              </button>
            </div>
          ) : (
            <>
              <select
                className="scene-select"
                value={selectedSceneId ?? ''}
                onChange={e => {
                  const value = e.target.value;
                  if (value === 'new') {
                    setIsCreatingScene(true);
                  } else if (value === '') {
                    setSelectedSceneId(null);
                  } else {
                    setSelectedSceneId(Number(value));
                  }
                }}
              >
                {scenes.length === 0 ? (
                  <option value="">Select a scene...</option>
                ) : (
                  <>
                    {scenes.map(scene => (
                      <option key={scene.id} value={scene.id}>
                        {`${scene.name} (ID: ${scene.id})`}
                      </option>
                    ))}
                  </>
                )}
                <option value="new">&lt;new...&gt;</option>
              </select>
              <button
                title="Manage scenes"
                onClick={() => setIsSceneManagementModalOpen(true)}
                className="scene-management-button"
              >
                ...
              </button>
            </>
          )}
        </div>
        <div className="objects-box">
          {shapes.map((shape, idx) => (
            <div
              key={shape.dbObject.name + idx}
              className={`object-line${shape.state === 'Selected' || shape.state === 'SuggestedSelected' ? ' object-line-selected' : ''}`}
              onClick={() => selectShape(shape)}
            >
              <span className="object-icon">{shape.getIcon()}</span>
              <span
                className="object-description"
                title={shape.getDescription()}
              >
                {shape.getDescription()}
              </span>
              {shape.dbObject.object_type === ObjectType.Locus && (
                <button
                  className="plot-toggle-button"
                  style={{
                    backgroundColor: displayedPlotNames.has(shape.dbObject.name)
                      ? PLOT_COLORS[getLocusOrdinal(shape.dbObject.name)]
                      : '#f5f5f5',
                    color: displayedPlotNames.has(shape.dbObject.name) ? 'white' : '#666',
                    borderColor: displayedPlotNames.has(shape.dbObject.name)
                      ? PLOT_COLORS[getLocusOrdinal(shape.dbObject.name)]
                      : '#ddd'
                  }}
                  title={displayedPlotNames.has(shape.dbObject.name) ? 'Hide plot' : 'Show plot'}
                  onClick={(e) => {
                    e.stopPropagation();
                    handleTogglePlot(shape.dbObject.name);
                  }}
                >
                  {displayedPlotNames.has(shape.dbObject.name) ? 'on' : 'off'}
                </button>
              )}
              <button
                className="delete-button"
                title="Delete"
                onClick={(e) => {
                  e.stopPropagation();
                  handleDeleteShape(shape);
                }}
              >
                ×
              </button>
            </div>
          ))}
        </div>
      </div>
      <div className="status-bar">{statusMessage}</div>
      <InvariantModal
        mousePos={mousePos}
        formula={invariantFormula}
        setFormula={setInvariantFormula}
        onSubmit={handleInvariantSubmit}
        onCancel={unsetAction}
        visible={currentAction?.name === ActionType.Invariant}
      />
      <SceneManagementModal
        isOpen={isSceneManagementModalOpen}
        onClose={() => setIsSceneManagementModalOpen(false)}
        onSceneDeleted={async () => {
          // Refresh scenes and get the updated list
          const response = await fetch('http://localhost:8080/scenes');
          const updatedScenes: SceneInfo[] = await response.json();
          setScenes(updatedScenes);

          // Check if the currently selected scene still exists
          const currentSceneExists = updatedScenes.find(s => s.id === selectedSceneId);

          if (!currentSceneExists) {
            // Current scene was deleted, select the last available scene
            if (updatedScenes.length > 0) {
              const newSelectedId = updatedScenes[updatedScenes.length - 1].id;
              setSelectedSceneId(newSelectedId);
            } else {
              // No scenes left, clear selection
              setSelectedSceneId(null);
            }
          }
        }}
      />
      <Legend
        displayedPlotNames={displayedPlotNames}
        plotDataByLocusName={plotDataByLocusName}
        shapes={shapes}
      />
      <ConfirmationModal
        isOpen={confirmationModal.isOpen}
        title={confirmationModal.title}
        message={confirmationModal.message}
        dependents={confirmationModal.dependents}
        dependentsTitle="The following objects will also be deleted:"
        onConfirm={confirmationModal.onConfirm}
        onCancel={() => setConfirmationModal(prev => ({ ...prev, isOpen: false }))}
      />
    </div>
  );
}

export default App;
