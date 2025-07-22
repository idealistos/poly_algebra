import { ActionType } from "./enums";
import GpsFixedIcon from '@mui/icons-material/GpsFixed';
import RadioButtonCheckedIcon from '@mui/icons-material/RadioButtonChecked';

export function getActionIcon(action: ActionType) {
    switch (action) {
        case ActionType.FixedPoint:
            return <GpsFixedIcon fontSize="medium" />;
        case ActionType.FreePoint:
            return <RadioButtonCheckedIcon fontSize="medium" />;
        case ActionType.Midpoint:
            return (
                <svg width="24" height="24" viewBox="0 0 24 24" style={{ display: 'block' }}>
                    {/* Line connecting the points */}
                    <line x1="3" y1="14" x2="21" y2="10" stroke="currentColor" strokeWidth="1" strokeDasharray="2,2" />
                    {/* Left point (slightly lower) */}
                    <circle cx="3" cy="14" r="2" stroke="currentColor" strokeWidth="1.5" fill="gray" />
                    {/* Right point (slightly higher) */}
                    <circle cx="21" cy="10" r="2" stroke="currentColor" strokeWidth="1.5" fill="gray" />
                    <circle cx="12" cy="12" r="2" fill="black" />
                </svg>
            );
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
        case ActionType.Projection:
            return (
                <svg width="24" height="24" viewBox="0 0 24 24" style={{ display: 'block' }}>
                    {/* Dotted horizontal line */}
                    <line x1="2" y1="20" x2="22" y2="20" stroke="currentColor" strokeWidth="1" strokeDasharray="2,2" />
                    {/* Dotted perpendicular line from point to line */}
                    <line x1="12" y1="4" x2="12" y2="20" stroke="currentColor" strokeWidth="1" strokeDasharray="2,2" />
                    {/* Non-filled circle (top point) */}
                    <circle cx="12" cy="4" r="2" stroke="currentColor" strokeWidth="1.5" fill="gray" />
                    {/* Filled circle (projected point on the line) */}
                    <circle cx="12" cy="20" r="2" fill="black" />
                </svg>
            );
        case ActionType.Reflection:
            return (
                <svg width="24" height="24" viewBox="0 0 24 24" style={{ display: 'block' }}>
                    {/* Dotted horizontal line */}
                    <line x1="2" y1="12" x2="22" y2="12" stroke="currentColor" strokeWidth="1" strokeDasharray="2,2" />
                    {/* Dotted perpendicular line from point to line */}
                    <line x1="12" y1="4" x2="12" y2="20" stroke="currentColor" strokeWidth="1" strokeDasharray="2,2" />
                    {/* Non-filled circle (top point) */}
                    <circle cx="12" cy="4" r="2" stroke="currentColor" strokeWidth="1.5" fill="gray" />
                    {/* Filled circle (reflected point) */}
                    <circle cx="12" cy="20" r="2" fill="black" />
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
        case ActionType.PpBisector:
            return (
                <svg width="24" height="24" viewBox="0 0 24 24" style={{ display: 'block' }}>
                    {/* Two dots on the same level */}
                    <circle cx="2" cy="12" r="2" fill="currentColor" />
                    <circle cx="22" cy="12" r="2" fill="currentColor" />
                    {/* Horizontal dotted line joining the two points */}
                    <line x1="4" y1="12" x2="20" y2="12" stroke="currentColor" strokeWidth="1" strokeDasharray="2,2" />
                    {/* Solid vertical line cutting the segment in the middle */}
                    <line x1="12" y1="6" x2="12" y2="18" stroke="currentColor" strokeWidth="2" />
                </svg>
            );
        case ActionType.PpToLine:
            return (
                <svg width="24" height="24" viewBox="0 0 24 24" style={{ display: 'block' }}>
                    {/* Thin dotted horizontal line in the lower part */}
                    <line x1="3" y1="18" x2="21" y2="18" stroke="currentColor" strokeWidth="1" strokeDasharray="2,2" />
                    {/* Solid thick line perpendicular to the horizontal line */}
                    <line x1="12" y1="6" x2="12" y2="24" stroke="currentColor" strokeWidth="2" />
                    {/* A circle representing a point on the perpendicular */}
                    <circle cx="12" cy="12" r="2" stroke="currentColor" strokeWidth="1.5" fill="gray" />
                </svg>
            );
        case ActionType.PlToLine:
            return (
                <svg width="24" height="24" viewBox="0 0 24 24" style={{ display: 'block' }}>
                    {/* Thin dotted horizontal line in the lower part */}
                    <line x1="3" y1="18" x2="21" y2="18" stroke="currentColor" strokeWidth="1" strokeDasharray="2,2" />
                    {/* Solid thick horizontal line through the point */}
                    <line x1="2" y1="12" x2="22" y2="12" stroke="currentColor" strokeWidth="2" />
                    {/* A circle representing a point on the horizontal line */}
                    <circle cx="12" cy="12" r="2" stroke="currentColor" strokeWidth="1.5" fill="gray" />
                </svg>
            );
        case ActionType.Parameter:
            return (
                <span className="parameter-icon">
                    p
                </span>
            );
        case ActionType.DistanceInvariant:
            return (
                <span className="invariant-icon">
                    d(•,•)
                </span>
            );
        case ActionType.AngleInvariant:
            return (
                <span className="invariant-icon">
                    α(•,•)
                </span>
            );
        case ActionType.Invariant:
            return (
                <span className="invariant-icon">
                    *
                </span>
            );
        case ActionType.Locus:
            return (
                <span className="locus-icon">
                    ∞
                </span>
            );
        default: {
            const exhaustiveCheck: never = action;
            throw new Error(`Unhandled action type: ${exhaustiveCheck}`);
        }
    }
} 