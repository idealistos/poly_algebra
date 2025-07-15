import React, { useState, useEffect } from 'react';
import './SceneManagementModal.css';

interface SceneInfo {
    id: number;
    name: string;
    created_at: string;
}

interface SceneManagementModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSceneDeleted: () => void;
}

export function SceneManagementModal({ isOpen, onClose, onSceneDeleted }: SceneManagementModalProps) {
    const [scenes, setScenes] = useState<SceneInfo[]>([]);
    const [selectedScenes, setSelectedScenes] = useState<Set<number>>(new Set());
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // Fetch scenes when modal opens
    useEffect(() => {
        if (isOpen) {
            fetchScenes();
        }
    }, [isOpen]);

    const fetchScenes = async () => {
        try {
            setIsLoading(true);
            setError(null);
            const response = await fetch('http://localhost:8080/scenes');
            if (!response.ok) {
                throw new Error(`Failed to fetch scenes: ${response.statusText}`);
            }
            const sceneInfos: SceneInfo[] = await response.json();
            setScenes(sceneInfos);
        } catch (err) {
            console.error('Failed to fetch scenes:', err);
            setError(err instanceof Error ? err.message : 'Unknown error occurred');
        } finally {
            setIsLoading(false);
        }
    };

    const handleSceneToggle = (sceneId: number) => {
        setSelectedScenes(prev => {
            const newSet = new Set(prev);
            if (newSet.has(sceneId)) {
                newSet.delete(sceneId);
            } else {
                newSet.add(sceneId);
            }
            return newSet;
        });
    };

    const handleSelectAll = () => {
        if (selectedScenes.size === scenes.length) {
            // If all are selected, deselect all
            setSelectedScenes(new Set());
        } else {
            // Select all
            setSelectedScenes(new Set(scenes.map(scene => scene.id)));
        }
    };

    const handleDeleteSelected = async () => {
        if (selectedScenes.size === 0) return;

        const sceneNames = scenes
            .filter(scene => selectedScenes.has(scene.id))
            .map(scene => scene.name);

        const confirmMessage = `Are you sure you want to delete ${selectedScenes.size} scene(s):\n${sceneNames.join(', ')}?`;

        if (!window.confirm(confirmMessage)) {
            return;
        }

        try {
            setIsLoading(true);
            setError(null);

            // Delete each selected scene
            const deletePromises = Array.from(selectedScenes).map(async (sceneId) => {
                const response = await fetch(`http://localhost:8080/scenes/${sceneId}`, {
                    method: 'DELETE',
                });
                if (!response.ok) {
                    throw new Error(`Failed to delete scene ${sceneId}: ${response.statusText}`);
                }
            });

            await Promise.all(deletePromises);

            // Clear selections and refresh scenes
            setSelectedScenes(new Set());
            await fetchScenes();
            onSceneDeleted();
        } catch (err) {
            console.error('Failed to delete scenes:', err);
            setError(err instanceof Error ? err.message : 'Unknown error occurred');
        } finally {
            setIsLoading(false);
        }
    };

    if (!isOpen) return null;

    return (
        <div className="scene-management-modal-overlay" onClick={onClose}>
            <div className="scene-management-modal-content" onClick={e => e.stopPropagation()}>
                <div className="scene-management-modal-title">Manage Scenes</div>

                {error && (
                    <div className="scene-management-error-message">
                        {error}
                    </div>
                )}

                {isLoading && scenes.length === 0 ? (
                    <div className="scene-management-loading">Loading scenes...</div>
                ) : (
                    <>
                        <div className="scene-management-list-header">
                            <div>
                                <input
                                    type="checkbox"
                                    className="scene-management-checkbox"
                                    checked={selectedScenes.size === scenes.length && scenes.length > 0}
                                    onChange={handleSelectAll}
                                />
                            </div>
                            <div>ID</div>
                            <div>Name</div>
                            <div>Created At</div>
                        </div>

                        <div className="scene-management-list">
                            {scenes.map(scene => (
                                <div
                                    key={scene.id}
                                    className="scene-management-item"
                                >
                                    <div>
                                        <input
                                            type="checkbox"
                                            className="scene-management-checkbox"
                                            checked={selectedScenes.has(scene.id)}
                                            onChange={() => handleSceneToggle(scene.id)}
                                        />
                                    </div>
                                    <div>{scene.id}</div>
                                    <div>{scene.name}</div>
                                    <div>{new Date(scene.created_at).toLocaleDateString()}</div>
                                </div>
                            ))}
                        </div>

                        <div className="scene-management-buttons">
                            <button
                                className="scene-management-close-button"
                                onClick={onClose}
                                disabled={isLoading}
                            >
                                Close
                            </button>
                            <button
                                className="scene-management-delete-button"
                                onClick={handleDeleteSelected}
                                disabled={selectedScenes.size === 0 || isLoading}
                            >
                                {isLoading ? 'Deleting...' : `Delete Selected (${selectedScenes.size})`}
                            </button>
                        </div>
                    </>
                )}
            </div>
        </div>
    );
} 