import React, { useState, useEffect, useCallback } from 'react';
import { createPortal } from 'react-dom';
import './GuidedTour.css';

export interface TourStep {
  id: string;
  target: string; // CSS selector
  title: string;
  content: string;
  placement?: 'top' | 'bottom' | 'left' | 'right';
  action?: string; // Optional action text
}

interface GuidedTourProps {
  steps: TourStep[];
  isOpen: boolean;
  onClose: () => void;
  onComplete?: () => void;
  tourId: string;
}

export const GuidedTour: React.FC<GuidedTourProps> = ({
  steps,
  isOpen,
  onClose,
  onComplete,
  tourId,
}) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [targetElement, setTargetElement] = useState<HTMLElement | null>(null);

  const step = steps[currentStep];

  const calculatePosition = useCallback(() => {
    if (!step) return;

    const element = document.querySelector(step.target) as HTMLElement;
    setTargetElement(element);

    if (!element) {
      // Center in viewport if target not found
      setPosition({
        x: window.innerWidth / 2 - 150,
        y: window.innerHeight / 2 - 100,
      });
      return;
    }

    const rect = element.getBoundingClientRect();
    const placement = step.placement || 'bottom';
    const tooltipWidth = 300;
    const tooltipHeight = 150;
    const offset = 16;

    let x = 0;
    let y = 0;

    switch (placement) {
      case 'top':
        x = rect.left + rect.width / 2 - tooltipWidth / 2;
        y = rect.top - tooltipHeight - offset;
        break;
      case 'bottom':
        x = rect.left + rect.width / 2 - tooltipWidth / 2;
        y = rect.bottom + offset;
        break;
      case 'left':
        x = rect.left - tooltipWidth - offset;
        y = rect.top + rect.height / 2 - tooltipHeight / 2;
        break;
      case 'right':
        x = rect.right + offset;
        y = rect.top + rect.height / 2 - tooltipHeight / 2;
        break;
    }

    // Boundary checks
    const padding = 16;
    x = Math.max(padding, Math.min(x, window.innerWidth - tooltipWidth - padding));
    y = Math.max(padding, Math.min(y, window.innerHeight - tooltipHeight - padding));

    setPosition({ x, y });
  }, [step]);

  useEffect(() => {
    if (isOpen) {
      calculatePosition();
      window.addEventListener('resize', calculatePosition);
      window.addEventListener('scroll', calculatePosition, true);
    }

    return () => {
      window.removeEventListener('resize', calculatePosition);
      window.removeEventListener('scroll', calculatePosition, true);
    };
  }, [isOpen, calculatePosition]);

  useEffect(() => {
    if (isOpen && targetElement) {
      // Highlight target element
      targetElement.classList.add('tour-highlight');
      targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' });

      return () => {
        targetElement.classList.remove('tour-highlight');
      };
    }
  }, [isOpen, targetElement, currentStep]);

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      handleComplete();
    }
  };

  const handlePrevious = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSkip = () => {
    // Save to localStorage that user skipped this tour
    localStorage.setItem(`tour_${tourId}_skipped`, 'true');
    onClose();
  };

  const handleComplete = () => {
    // Save to localStorage that user completed this tour
    localStorage.setItem(`tour_${tourId}_completed`, 'true');
    onComplete?.();
    onClose();
  };

  const handleStepClick = (index: number) => {
    setCurrentStep(index);
  };

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return;

      switch (e.key) {
        case 'ArrowRight':
        case ' ':
          e.preventDefault();
          handleNext();
          break;
        case 'ArrowLeft':
          e.preventDefault();
          handlePrevious();
          break;
        case 'Escape':
          e.preventDefault();
          handleSkip();
          break;
        case 'Home':
          e.preventDefault();
          setCurrentStep(0);
          break;
        case 'End':
          e.preventDefault();
          setCurrentStep(steps.length - 1);
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, currentStep, steps.length]);

  if (!isOpen || !step) return null;

  const progress = ((currentStep + 1) / steps.length) * 100;

  return createPortal(
    <>
      {/* Overlay */}
      <div className="tour-overlay" onClick={handleSkip} />

      {/* Spotlight for target element */}
      {targetElement && (
        <div
          className="tour-spotlight"
          style={{
            position: 'fixed',
            top: targetElement.getBoundingClientRect().top - 8,
            left: targetElement.getBoundingClientRect().left - 8,
            width: targetElement.getBoundingClientRect().width + 16,
            height: targetElement.getBoundingClientRect().height + 16,
          }}
        />
      )}

      {/* Tour Card */}
      <div
        className={`tour-card tour-${step.placement || 'bottom'}`}
        style={{
          left: position.x,
          top: position.y,
        }}
      >
        {/* Progress Bar */}
        <div className="tour-progress">
          <div
            className="tour-progress-bar"
            style={{ width: `${progress}%` }}
          />
        </div>

        {/* Header */}
        <div className="tour-header">
          <span className="tour-badge">Tour</span>
          <span className="tour-step-count">
            Step {currentStep + 1} of {steps.length}
          </span>
        </div>

        {/* Content */}
        <div className="tour-content">
          <h3>{step.title}</h3>
          <p>{step.content}</p>
          {step.action && (
            <div className="tour-action">
              <span className="tour-action-label">Try this:</span>
              <span className="tour-action-text">{step.action}</span>
            </div>
          )}
        </div>

        {/* Step Indicators */}
        <div className="tour-indicators">
          {steps.map((_, index) => (
            <button
              key={index}
              className={`tour-indicator ${index === currentStep ? 'active' : ''} ${
                index < currentStep ? 'completed' : ''
              }`}
              onClick={() => handleStepClick(index)}
              aria-label={`Go to step ${index + 1}`}
            />
          ))}
        </div>

        {/* Footer */}
        <div className="tour-footer">
          <button className="tour-skip" onClick={handleSkip}>
            Skip Tour
          </button>
          <div className="tour-nav">
            <button
              onClick={handlePrevious}
              disabled={currentStep === 0}
              className="tour-prev"
            >
              ← Back
            </button>
            <button onClick={handleNext} className="tour-next">
              {currentStep === steps.length - 1 ? 'Finish' : 'Next →'}
            </button>
          </div>
        </div>
      </div>
    </>,
    document.body
  );
};

// Hook to manage tour state
export const useTour = (tourId: string) => {
  const [hasCompleted, setHasCompleted] = useState(() => {
    return localStorage.getItem(`tour_${tourId}_completed`) === 'true';
  });

  const [hasSkipped, setHasSkipped] = useState(() => {
    return localStorage.getItem(`tour_${tourId}_skipped`) === 'true';
  });

  const resetTour = () => {
    localStorage.removeItem(`tour_${tourId}_completed`);
    localStorage.removeItem(`tour_${tourId}_skipped`);
    setHasCompleted(false);
    setHasSkipped(false);
  };

  return {
    hasCompleted,
    hasSkipped,
    shouldShow: !hasCompleted && !hasSkipped,
    resetTour,
  };
};

// Predefined tours
export const NEW_USER_TOUR: TourStep[] = [
  {
    id: 'welcome',
    target: '.app-container',
    title: 'Welcome to SPO!! Editor',
    content:
      'This is the Super Punch-Out!! Editor. Let\'s take a quick tour to help you get started with editing your favorite SNES game.',
    placement: 'bottom',
  },
  {
    id: 'open-rom',
    target: 'button[onClick*="handleOpenRom"]',
    title: 'Open a ROM',
    content:
      'Start by opening your Super Punch-Out!! ROM file. The editor supports .sfc and .smc formats.',
    placement: 'bottom',
  },
  {
    id: 'boxer-list',
    target: '.boxer-list',
    title: 'Select a Boxer',
    content:
      'Once your ROM is loaded, select a boxer from the list to start editing their sprites, palettes, or stats.',
    placement: 'right',
  },
  {
    id: 'palette-editor',
    target: '.palette-editor',
    title: 'Edit Palettes',
    content:
      'Use the Palette Editor to change the colors used by fighters. Click any color to modify it.',
    placement: 'left',
  },
  {
    id: 'export',
    target: '.export-panel',
    title: 'Export Your Changes',
    content:
      'When you\'re done editing, export your changes as an IPS patch to share with others!',
    placement: 'top',
  },
];

export const PALETTE_EDITOR_TOUR: TourStep[] = [
  {
    id: 'palette-intro',
    target: '.palette-editor',
    title: 'Palette Editor',
    content:
      'The Palette Editor lets you modify the colors used by fighters in the game.',
    placement: 'right',
  },
  {
    id: 'color-grid',
    target: '.color-grid',
    title: 'Color Grid',
    content: 'Click any color in the grid to select it for editing.',
    placement: 'right',
  },
  {
    id: 'color-picker',
    target: '.color-picker',
    title: 'Color Picker',
    content: 'Use the color picker to adjust the RGB values or enter a hex code.',
    placement: 'left',
  },
];

export default GuidedTour;
