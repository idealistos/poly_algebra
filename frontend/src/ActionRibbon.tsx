import { useEffect, useState, useRef } from 'react';
import { IconButton, Tooltip, Box, Paper } from '@mui/material';
import { ActionType, ObjectType } from './enums';
import type { Action } from './types';
import './ActionRibbon.css';
import { getActionIcon } from './actionIcons';

interface GroupedActions {
  [key: string]: Action[];
}

export function ActionRibbon({
  onActionClick,
  setStatusMessage,
  setClickedActionButtonCorner
}: {
  onActionClick: (action: Action) => void,
  setStatusMessage: (msg: string | null) => void,
  setClickedActionButtonCorner: (corner: { x: number, y: number } | null) => void
}) {
  const [groupedActions, setGroupedActions] = useState<GroupedActions>({});
  const [expandedGroup, setExpandedGroup] = useState<string | null>(null);
  const [showTooltip, setShowTooltip] = useState<string | null>(null);
  const hoverTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

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
              action.object_types !== undefined && action.object_types.length > 0 &&
              action.object_types.every(isValidObjectType);
          }
        );
        // Group actions by their group field
        const grouped: GroupedActions = {};
        validActions.forEach(action => {
          if (!grouped[action.group]) {
            grouped[action.group] = [];
          }
          grouped[action.group].push(action);
        });
        setGroupedActions(grouped);
      })
      .catch(() => setGroupedActions({}));
  }, []);

  const handleButtonClick = (action: Action, event?: React.MouseEvent<HTMLButtonElement>) => {
    setStatusMessage(action.arguments[0]?.hint ?? null);
    onActionClick(action);
    setExpandedGroup(null); // Close expansion after selection

    // Capture the button's top-left corner position
    if (event) {
      const buttonElement = event.currentTarget;
      const rect = buttonElement.getBoundingClientRect();
      setClickedActionButtonCorner({
        x: rect.left,
        y: rect.top
      });
    }
  };

  const handleGroupHover = (groupKey: string, hasMultipleActions: boolean) => {
    // Show tooltip initially for all groups
    setShowTooltip(groupKey);

    if (!hasMultipleActions) return;

    // Clear any existing timer
    if (hoverTimeoutRef.current) {
      clearTimeout(hoverTimeoutRef.current);
    }

    // Set a 1-second timer before expanding
    hoverTimeoutRef.current = setTimeout(() => {
      setExpandedGroup(groupKey);
      setShowTooltip(null); // Hide tooltip when group expands
    }, 1000);
  };

  const handleMoreActionsHover = (groupKey: string) => {
    // Clear any existing timer
    if (hoverTimeoutRef.current) {
      clearTimeout(hoverTimeoutRef.current);
      hoverTimeoutRef.current = null;
    }
    // Expand immediately and hide tooltip
    setExpandedGroup(groupKey);
    setShowTooltip(null);
  };

  const handleGroupLeave = () => {
    setExpandedGroup(null);
    setShowTooltip(null);
    if (hoverTimeoutRef.current) {
      clearTimeout(hoverTimeoutRef.current);
      hoverTimeoutRef.current = null;
    }
  };

  // Get the first action from each group (default icon)
  const getDefaultActions = () => {
    return Object.entries(groupedActions).map(([groupKey, groupActions]) => ({
      groupKey,
      defaultAction: groupActions[0],
      hasMultipleActions: groupActions.length > 1
    }));
  };

  return (
    <Box className="action-ribbon-container">
      <Paper
        elevation={3}
        className="action-ribbon-paper"
      >
        {getDefaultActions().map(({ groupKey, defaultAction, hasMultipleActions }) => (
          <Box
            key={groupKey}
            className="action-group"
            onMouseEnter={() => handleGroupHover(groupKey, hasMultipleActions)}
            onMouseLeave={handleGroupLeave}
          >
            <Tooltip
              title={defaultAction.description}
              placement="right"
              open={showTooltip === groupKey && expandedGroup !== groupKey}
            >
              <span>
                <IconButton
                  size="large"
                  onClick={(event) => handleButtonClick(defaultAction, event)}
                >
                  {getActionIcon(defaultAction.name)}
                </IconButton>
              </span>
            </Tooltip>

            {hasMultipleActions && (
              <Box
                className="more-actions-indicator"
                onMouseEnter={() => handleMoreActionsHover(groupKey)}
              >
                +{groupedActions[groupKey].length - 1}...
              </Box>
            )}

            {expandedGroup === groupKey && hasMultipleActions && (
              <Box className="expanded-actions">
                {groupedActions[groupKey].map((action) => (
                  <Tooltip key={action.name} title={action.description} placement="right">
                    <span>
                      <IconButton
                        size="large"
                        onClick={(event) => handleButtonClick(action, event)}
                      >
                        {getActionIcon(action.name)}
                      </IconButton>
                    </span>
                  </Tooltip>
                ))}
              </Box>
            )}
          </Box>
        ))}
      </Paper>
    </Box>
  );
} 