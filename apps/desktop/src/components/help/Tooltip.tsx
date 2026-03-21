import React, { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import './Tooltip.css';

interface TooltipProps {
  children: React.ReactNode;
  content: React.ReactNode;
  placement?: 'top' | 'bottom' | 'left' | 'right';
  delay?: number;
  maxWidth?: number;
  disabled?: boolean;
  className?: string;
}

export const Tooltip: React.FC<TooltipProps> = ({
  children,
  content,
  placement = 'top',
  delay = 300,
  maxWidth = 280,
  disabled = false,
  className = '',
}) => {
  const [isVisible, setIsVisible] = useState(false);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const triggerRef = useRef<HTMLDivElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const showTooltip = () => {
    if (disabled) return;
    
    timeoutRef.current = setTimeout(() => {
      if (triggerRef.current) {
        const rect = triggerRef.current.getBoundingClientRect();
        const tooltipRect = tooltipRef.current?.getBoundingClientRect();
        const tooltipHeight = tooltipRect?.height || 40;
        const tooltipWidth = tooltipRect?.width || 200;

        let x = 0;
        let y = 0;

        switch (placement) {
          case 'top':
            x = rect.left + rect.width / 2;
            y = rect.top - tooltipHeight - 8;
            break;
          case 'bottom':
            x = rect.left + rect.width / 2;
            y = rect.bottom + 8;
            break;
          case 'left':
            x = rect.left - tooltipWidth - 8;
            y = rect.top + rect.height / 2;
            break;
          case 'right':
            x = rect.right + 8;
            y = rect.top + rect.height / 2;
            break;
        }

        // Boundary checks
        const padding = 8;
        x = Math.max(padding, Math.min(x, window.innerWidth - tooltipWidth - padding));
        y = Math.max(padding, Math.min(y, window.innerHeight - tooltipHeight - padding));

        setPosition({ x, y });
        setIsVisible(true);
      }
    }, delay);
  };

  const hideTooltip = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setIsVisible(false);
  };

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return (
    <>
      <div
        ref={triggerRef}
        className={`tooltip-trigger ${className}`}
        onMouseEnter={showTooltip}
        onMouseLeave={hideTooltip}
        onFocus={showTooltip}
        onBlur={hideTooltip}
      >
        {children}
      </div>

      {isVisible &&
        createPortal(
          <div
            ref={tooltipRef}
            className={`tooltip tooltip-${placement}`}
            style={{
              left: position.x,
              top: position.y,
              maxWidth,
            }}
            onMouseEnter={() => setIsVisible(true)}
            onMouseLeave={hideTooltip}
          >
            <div className="tooltip-content">{content}</div>
            <div className="tooltip-arrow" />
          </div>,
          document.body
        )}
    </>
  );
};

// Rich tooltip with title and content
interface RichTooltipProps extends Omit<TooltipProps, 'content'> {
  title?: string;
  description: string;
  shortcut?: string;
  link?: {
    text: string;
    onClick: () => void;
  };
}

export const RichTooltip: React.FC<RichTooltipProps> = ({
  title,
  description,
  shortcut,
  link,
  children,
  ...props
}) => {
  const content = (
    <div className="rich-tooltip">
      {title && <div className="rich-tooltip-title">{title}</div>}
      <div className="rich-tooltip-description">{description}</div>
      {shortcut && (
        <div className="rich-tooltip-shortcut">
          <kbd>{shortcut}</kbd>
        </div>
      )}
      {link && (
        <button className="rich-tooltip-link" onClick={link.onClick}>
          {link.text} →
        </button>
      )}
    </div>
  );

  return (
    <Tooltip content={content} {...props}>
      {children}
    </Tooltip>
  );
};

// Info tooltip - simple help text
interface InfoTooltipProps extends Omit<TooltipProps, 'content'> {
  text: string;
}

export const InfoTooltip: React.FC<InfoTooltipProps> = ({ text, children, ...props }) => {
  return (
    <Tooltip content={<span className="info-tooltip-text">{text}</span>} {...props}>
      {children}
    </Tooltip>
  );
};

export default Tooltip;
