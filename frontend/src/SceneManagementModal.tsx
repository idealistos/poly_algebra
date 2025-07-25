import { useState, useEffect } from 'react';
import './SceneManagementModal.css';
import { ConfirmationModal } from './ConfirmationModal';

interface SceneInfo {
    id: number;
    name: string;
    created_at: string;
}

interface SceneManagementModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSceneDeleted: () => void;
    onSceneSelected?: (sceneId: number) => void;
}

export function SceneManagementModal({ isOpen, onClose, onSceneDeleted, onSceneSelected }: SceneManagementModalProps) {
    const [scenes, setScenes] = useState<SceneInfo[]>([]);
    const [selectedScenes, setSelectedScenes] = useState<Set<number>>(new Set());
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [editingSceneId, setEditingSceneId] = useState<number | null>(null);
    const [editingName, setEditingName] = useState<string>('');
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

    const handleStartEdit = (scene: SceneInfo) => {
        setEditingSceneId(scene.id);
        setEditingName(scene.name);
    };

    const handleCancelEdit = () => {
        setEditingSceneId(null);
        setEditingName('');
    };

    const handleSaveEdit = async () => {
        if (!editingSceneId || !editingName.trim()) {
            handleCancelEdit();
            return;
        }

        try {
            setIsLoading(true);
            setError(null);

            const response = await fetch(`http://localhost:8080/scenes/${editingSceneId}`, {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ name: editingName.trim() }),
            });
            console.log(response);

            if (!response.ok) {
                throw new Error(`Failed to rename scene: ${response.statusText}`);
            }

            // Update the scene in the local state
            setScenes(prev => prev.map(scene =>
                scene.id === editingSceneId
                    ? { ...scene, name: editingName.trim() }
                    : scene
            ));

            handleCancelEdit();
        } catch (err) {
            console.error('Failed to rename scene:', err);
            setError(err instanceof Error ? err.message : 'Unknown error occurred');
        } finally {
            setIsLoading(false);
        }
    };

    const handleKeyPress = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter') {
            handleSaveEdit();
        } else if (e.key === 'Escape') {
            handleCancelEdit();
        }
    };

    const handleSceneClick = (sceneId: number) => {
        if (onSceneSelected) {
            onSceneSelected(sceneId);
            onClose();
        }
    };

    const handleDeleteSelected = async () => {
        if (selectedScenes.size === 0) return;

        const sceneNames = scenes
            .filter(scene => selectedScenes.has(scene.id))
            .map(scene => scene.name);

        // Show confirmation modal
        setConfirmationModal({
            isOpen: true,
            title: 'Delete Scenes',
            message: `Are you sure you want to delete ${selectedScenes.size} scene(s)?`,
            dependents: sceneNames,
            onConfirm: async () => {
                // Close the modal
                setConfirmationModal(prev => ({ ...prev, isOpen: false }));

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
            }
        });
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
                                    <div className="scene-name-cell">
                                        {editingSceneId === scene.id ? (
                                            <input
                                                type="text"
                                                value={editingName}
                                                onChange={(e) => setEditingName(e.target.value)}
                                                onKeyDown={handleKeyPress}
                                                onBlur={handleSaveEdit}
                                                autoFocus
                                                className="scene-name-input"
                                            />
                                        ) : (
                                            <div className="scene-name-display">
                                                <button
                                                    className="scene-name-link"
                                                    onClick={() => handleSceneClick(scene.id)}
                                                    title="Select this scene"
                                                >
                                                    {scene.name}
                                                </button>
                                                <button
                                                    className="scene-edit-button"
                                                    onClick={() => handleStartEdit(scene)}
                                                    title="Edit scene name"
                                                >
                                                    ✏️
                                                </button>
                                            </div>
                                        )}
                                    </div>
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
            <ConfirmationModal
                isOpen={confirmationModal.isOpen}
                title={confirmationModal.title}
                message={confirmationModal.message}
                dependents={confirmationModal.dependents}
                dependentsTitle="List of deleted scenes:"
                onConfirm={confirmationModal.onConfirm}
                onCancel={() => setConfirmationModal(prev => ({ ...prev, isOpen: false }))}
            />
        </div>
    );
} 