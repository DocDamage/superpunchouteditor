import React, { useState } from 'react';
import { HelpSystem } from './HelpSystem';
import './HelpButton.css';

interface HelpButtonProps {
  context?: string;
  articleId?: string;
  className?: string;
  size?: 'small' | 'medium' | 'large';
  variant?: 'icon' | 'button' | 'link';
  label?: string;
  style?: React.CSSProperties;
}

export const HelpButton: React.FC<HelpButtonProps> = ({
  context,
  articleId,
  className = '',
  size = 'medium',
  variant = 'icon',
  label,
  style,
}) => {
  const [isHelpOpen, setIsHelpOpen] = useState(false);

  const handleClick = () => {
    setIsHelpOpen(true);
  };

  const sizeClasses = {
    small: 'help-btn-small',
    medium: 'help-btn-medium',
    large: 'help-btn-large',
  };

  const variantClasses = {
    icon: 'help-btn-icon',
    button: 'help-btn-button',
    link: 'help-btn-link',
  };

  return (
    <>
      <button
        className={`help-button ${sizeClasses[size]} ${variantClasses[variant]} ${className}`}
        onClick={handleClick}
        title="Get help"
        aria-label="Open help"
        style={style}
      >
        {variant === 'icon' ? (
          <span className="help-icon">?</span>
        ) : variant === 'link' ? (
          <>
            <span className="help-icon">?</span>
            <span className="help-label">{label || 'Help'}</span>
          </>
        ) : (
          <>
            <span className="help-icon">?</span>
            <span className="help-label">{label || 'Help'}</span>
          </>
        )}
      </button>

      <HelpSystem
        isOpen={isHelpOpen}
        onClose={() => setIsHelpOpen(false)}
        initialArticleId={articleId}
        initialContext={context}
      />
    </>
  );
};

export default HelpButton;
