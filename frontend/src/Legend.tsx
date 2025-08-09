import React from 'react';
import type { PlotData, Shape } from './types';
import { PLOT_COLORS } from './utils';
import { ObjectType } from './enums';
import './Legend.css';

interface LegendProps {
    displayedPlotNames: Set<string>;
    plotDataByLocusName: Record<string, PlotData>;
    shapes: Shape[];
}

// Function to process equation text for better line breaking
function processEquationForWrapping(equation: string): string {
    // Add non-breaking spaces after operators to prevent breaking within monomials
    console.log(equation.replace(/([+\-=])\s/g, '$1\u00A0'))
    return equation.replace(/([+\-=])\s/g, '$1\u00A0').replace(/-/g, '\u2212');
}

function Legend({ displayedPlotNames, plotDataByLocusName, shapes }: LegendProps) {
    // If no plots are displayed, don't render anything
    if (displayedPlotNames.size === 0) {
        return null;
    }

    // Helper function to get locus ordinal number
    const getLocusOrdinal = (locusName: string) => {
        const locusShapes = shapes.filter(shape =>
            shape.objectType === ObjectType.Locus
        );
        return locusShapes.findIndex(shape => shape.name === locusName) % 10;
    };

    return (
        <div className="legend">
            <div className="legend-title">Curves</div>
            {Array.from(displayedPlotNames).map(locusName => {
                const plotData = plotDataByLocusName[locusName];
                if (!plotData) return null;

                const locusOrdinal = getLocusOrdinal(locusName);
                const color = PLOT_COLORS[locusOrdinal];

                // If there are multiple factors, create separate legend entries for each
                if (plotData.formatted_equations.length > 1) {
                    return plotData.formatted_equations.map((equation, index) => (
                        <div key={`${locusName}-factor-${index}`} className="legend-item">
                            <div
                                className="legend-line"
                                style={{ backgroundColor: color }}
                            />
                            <div className="legend-equation">
                                {processEquationForWrapping(equation)}
                            </div>
                        </div>
                    ));
                } else {
                    // Single factor or fallback to equation
                    const equationText = plotData.formatted_equations.length > 0
                        ? plotData.formatted_equations[0]
                        : plotData.equation;

                    return (
                        <div key={locusName} className="legend-item">
                            <div
                                className="legend-line"
                                style={{ backgroundColor: color }}
                            />
                            <div className="legend-equation">
                                {processEquationForWrapping(equationText)}
                            </div>
                        </div>
                    );
                }
            })}
        </div>
    );
};

export default Legend; 