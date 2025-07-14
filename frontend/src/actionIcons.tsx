import { ActionType } from "./enums";
import GpsFixedIcon from '@mui/icons-material/GpsFixed';
import RadioButtonCheckedIcon from '@mui/icons-material/RadioButtonChecked';
import SsidChartIcon from '@mui/icons-material/SsidChart';

export function getActionIcon(action: ActionType) {
    switch (action) {
        case ActionType.FixedPoint:
            return <GpsFixedIcon fontSize="medium" />;
        case ActionType.FreePoint:
            return <RadioButtonCheckedIcon fontSize="medium" />;
        case ActionType.Midpoint:
            return (
                <svg width="24" height="24" viewBox="0 0 24 24" style={{ display: 'block' }}>
                    {/* Left point (slightly lower) */}
                    <circle cx="4" cy="14" r="2" fill="currentColor" />
                    {/* Right point (slightly higher) */}
                    <circle cx="20" cy="10" r="2" fill="currentColor" />
                    {/* Line connecting the points */}
                    <line x1="4" y1="14" x2="20" y2="10" stroke="currentColor" strokeWidth="1" />
                    <circle cx="12" cy="12" r="2" fill="currentColor" />
                </svg>
            );
        case ActionType.LineAB:
            return (
                <svg width="24" height="24" viewBox="0 0 24 24" style={{ display: 'block' }}>
                    {/* Left point (slightly lower) */}
                    <circle cx="4" cy="16" r="2" fill="currentColor" />
                    {/* Right point (slightly higher) */}
                    <circle cx="20" cy="8" r="2" fill="currentColor" />
                    {/* Line connecting the points */}
                    <line x1="6" y1="15" x2="18" y2="9" stroke="currentColor" strokeWidth="1" strokeDasharray="2,2" />
                </svg>
            );
        case ActionType.Parameter:
            return (
                <span className="parameter-icon">
                    p
                </span>
            );
        case ActionType.Invariant:
            // Letter I in a square
            return (
                <span className="invariant-icon">
                    =
                </span>
            );
        case ActionType.Locus:
            return <SsidChartIcon fontSize="medium" />;
        case ActionType.IntersectionPoint:
            return (
                <svg width="24" height="24" viewBox="0 0 24 24" style={{ display: 'block' }}>
                    {/* X mark */}
                    <line x1="4" y1="4" x2="20" y2="20" stroke="currentColor" strokeWidth="2" />
                    <line x1="4" y1="20" x2="20" y2="4" stroke="currentColor" strokeWidth="2" />
                    {/* Filled circle */}
                    <circle cx="12" cy="12" r="3" fill="black" />
                </svg>
            );
        case ActionType.SlidingPoint:
            return (
                <svg width="24" height="24" viewBox="0 0 24 24" style={{ display: 'block' }}>
                    {/* Gray circle of radius 7 */}
                    <circle cx="12" cy="12" r="7" fill="gray" />
                    {/* Thick horizontal white line hiding the middle part */}
                    <rect x="1" y="10" width="22" height="4" fill="white" />
                    {/* Horizontal dotted line through the center */}
                    <line x1="1" y1="12" x2="23" y2="12" stroke="black" strokeWidth="1" strokeDasharray="2,2" />
                </svg>
            );
        default: {
            const exhaustiveCheck: never = action;
            throw new Error(`Unhandled action type: ${exhaustiveCheck}`);
        }
    }
} 