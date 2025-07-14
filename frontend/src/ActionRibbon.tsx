import { useEffect, useState } from 'react';
import { IconButton, Tooltip, Box, Paper } from '@mui/material';
import { ActionType, ObjectType } from './enums';
import type { Action } from './types';
import './ActionRibbon.css';
import { getActionIcon } from './actionIcons';

export function ActionRibbon({ onActionClick, setStatusMessage }: { onActionClick: (action: Action) => void, setStatusMessage: (msg: string | null) => void }) {
  const [actions, setActions] = useState<Action[]>([]);

  useEffect(() => {
    const isValidActionType = (value: string): value is ActionType => {
      return Object.values(ActionType).includes(value as ActionType);
    };

    const isValidObjectType = (value: string | null): value is ObjectType | null => {
      return value === null || Object.values(ObjectType).includes(value as ObjectType);
    };

    fetch('http://localhost:8080/actions')
      .then(res => res.json())
      .then((data: Partial<Action>[]) => {
        const validActions = data.filter(
          (action): action is Action => {
            return action.name !== undefined && isValidActionType(action.name) &&
              action.object_type !== undefined && isValidObjectType(action.object_type);
          }
        );
        setActions(validActions);
      })
      .catch(() => setActions([]));
  }, []);

  const handleButtonClick = (action: Action) => {
    setStatusMessage(action.arguments[0]?.hint ?? null);
    onActionClick(action);
  };

  return (
    <Box className="action-ribbon-container">
      <Paper
        elevation={3}
        className="action-ribbon-paper"
      >
        {actions.map((action) => (
          <Tooltip key={action.name} title={action.description} placement="right">
            <span>
              <IconButton size="large" onClick={() => handleButtonClick(action)}>
                {getActionIcon(action.name)}
              </IconButton>
            </span>
          </Tooltip>
        ))}
      </Paper>
    </Box>
  );
} 