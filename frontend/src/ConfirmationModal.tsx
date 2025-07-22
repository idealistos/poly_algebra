import React, { useEffect, useRef } from 'react';
import './ConfirmationModal.css';

interface ConfirmationModalProps {
    isOpen: boolean;
    title: string;
    message: string;
    dependents?: string[];
    dependentsTitle?: string;
    onConfirm: () => void;
    onCancel: () => void;
}

export function ConfirmationModal({
    isOpen,
    title,
    message,
    dependents,
    dependentsTitle,
    onConfirm,
    onCancel
}: ConfirmationModalProps) {
    const confirmButtonRef = useRef<HTMLButtonElement>(null);

    // Focus the confirm button when modal opens
    useEffect(() => {
        if (isOpen && confirmButtonRef.current) {
            confirmButtonRef.current.focus();
        }
    }, [isOpen]);

    // Handle keyboard events
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onCancel();
            } else if (e.key === 'Enter') {
                onConfirm();
            }
        };

        document.addEventListener('keydown', handleKeyDown);
        return () => document.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, onConfirm, onCancel]);

    if (!isOpen) return null;

    return (
        <div className="confirmation-modal-overlay" onClick={onCancel}>
            <div className="confirmation-modal-content" onClick={e => e.stopPropagation()}>
                <div className="confirmation-modal-title">{title}</div>

                <div className="confirmation-modal-message">{message}</div>

                {dependents && dependents.length > 0 && (
                    <div className="confirmation-modal-dependents">
                        <div className="confirmation-modal-dependents-title">
                            {dependentsTitle || "The following objects will also be deleted:"}
                        </div>
                        <div className="confirmation-modal-dependents-list">
                            {dependents.map((dependent, index) => (
                                <div key={index} className="confirmation-modal-dependent-item">
                                    • {dependent}
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                <div className="confirmation-modal-buttons">
                    <button
                        className="confirmation-modal-button confirmation-modal-button-cancel"
                        onClick={onCancel}
                    >
                        Cancel
                    </button>
                    <button
                        ref={confirmButtonRef}
                        className="confirmation-modal-button confirmation-modal-button-confirm"
                        onClick={onConfirm}
                    >
                        Delete
                    </button>
                </div>
            </div>
        </div>
    );
} 