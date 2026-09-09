/**
 * Text Preview Component
 * 
 * Shows how text will appear in-game with SPO's font style.
 * Provides a visual preview with proper formatting and layout.
 */

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './TextPreview.css';

interface TextPreviewProps {
  text: string;
  onClose: () => void;
  font?: 'default' | 'title' | 'small';
  maxWidth?: number;
}

interface PreviewResponse {
  rendered_text: string;
  line_count: number;
  fits_on_screen: boolean;
  estimated_width: number;
  estimated_height: number;
}

export function TextPreview({
  text,
  onClose,
  font = 'default',
  maxWidth = 28,
}: TextPreviewProps) {
  const [preview, setPreview] = useState<PreviewResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [fontSize, setFontSize] = useState(16);
  const [showGrid, setShowGrid] = useState(false);

  useEffect(() => {
    loadPreview();
  }, [text, font, maxWidth]);

  const loadPreview = async () => {
    if (!text) return;

    try {
      setLoading(true);
      const response = await invoke<PreviewResponse>('preview_text_render', {
        request: {
          text,
          font,
          maxWidth,
        },
      });
      setPreview(response);
    } catch (err) {
      console.error('Failed to load preview:', err);
      // Fallback to client-side rendering
      setPreview(renderClientSide(text, maxWidth));
    } finally {
      setLoading(false);
    }
  };

  // Client-side fallback preview
  const renderClientSide = (input: string, width: number): PreviewResponse => {
    const words = input.split(/\s+/);
    const lines: string[] = [];
    let currentLine = '';

    for (const word of words) {
      if (currentLine.length + word.length + 1 > width) {
        if (currentLine) {
          lines.push(currentLine);
        }
        currentLine = word;
      } else {
        currentLine = currentLine ? `${currentLine} ${word}` : word;
      }
    }

    if (currentLine) {
      lines.push(currentLine);
    }

    const renderedText = lines.join('\n');

    return {
      rendered_text: renderedText,
      line_count: lines.length,
      fits_on_screen: lines.length <= 3,
      estimated_width: Math.max(...lines.map((l) => l.length)) * 8,
      estimated_height: lines.length * 16,
    };
  };

  // Format text for display (convert control codes)
  const formatForDisplay = (input: string): string => {
    return input
      .replace(/\[END\]/gi, '')
      .replace(/\[BR\]/gi, '\n')
      .replace(/\[WAIT\]/gi, ' ⏸ ')
      .replace(/\[CLR\]/gi, '\n---\n')
      .replace(/\[COLOR:\d+\]/gi, '');
  };

  // Get font size label
  const getFontLabel = () => {
    switch (font) {
      case 'title':
        return 'Title Font (Large)';
      case 'small':
        return 'Small Font';
      default:
        return 'Default Font';
    }
  };

  // Get screen dimensions based on font
  const getScreenDimensions = () => {
    const baseWidth = 256;
    const baseHeight = 224;
    return { width: baseWidth, height: baseHeight };
  };

  const screen = getScreenDimensions();

  return (
    <div className="text-preview__overlay" onClick={onClose}>
      <div className="text-preview__modal" onClick={(e) => e.stopPropagation()}>
        <div className="text-preview__header">
          <h3>In-Game Preview</h3>
          <button className="text-preview__close" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="text-preview__controls">
          <div className="text-preview__control-group">
            <label>Font Size:</label>
            <input
              type="range"
              min={12}
              max={32}
              value={fontSize}
              onChange={(e) => setFontSize(Number(e.target.value))}
            />
            <span>{fontSize}px</span>
          </div>

          <div className="text-preview__control-group">
            <label>
              <input
                type="checkbox"
                checked={showGrid}
                onChange={(e) => setShowGrid(e.target.checked)}
              />
              Show Grid
            </label>
          </div>

          <div className="text-preview__info">
            <span className="text-preview__font-type">{getFontLabel()}</span>
            {preview && (
              <span
                className={`text-preview__fits-badge ${
                  preview.fits_on_screen
                    ? 'text-preview__fits-badge--ok'
                    : 'text-preview__fits-badge--warn'
                }`}
              >
                {preview.fits_on_screen ? '✓ Fits' : '⚠ Too Long'}
              </span>
            )}
          </div>
        </div>

        <div className="text-preview__display-area">
          {/* SNES-style text box background */}
          <div
            className="text-preview__snes-box"
            style={{
              width: screen.width,
              height: Math.min(screen.height, 120),
            }}
          >
            {showGrid && (
              <div className="text-preview__grid">
                {Array.from({ length: 20 }).map((_, i) => (
                  <div
                    key={`h-${i}`}
                    className="text-preview__grid-line text-preview__grid-line--horizontal"
                    style={{ top: `${(i + 1) * 5}%` }}
                  />
                ))}
                {Array.from({ length: 20 }).map((_, i) => (
                  <div
                    key={`v-${i}`}
                    className="text-preview__grid-line text-preview__grid-line--vertical"
                    style={{ left: `${(i + 1) * 5}%` }}
                  />
                ))}
              </div>
            )}

            <div
              className="text-preview__text"
              style={{
                fontSize: `${fontSize}px`,
                lineHeight: `${fontSize * 1.2}px`,
              }}
            >
              {loading ? (
                <span className="text-preview__loading">Loading preview...</span>
              ) : preview ? (
                formatForDisplay(preview.rendered_text)
                  .split('\n')
                  .map((line, i) => (
                    <div key={i} className="text-preview__line">
                      {line || '\u00A0'}
                    </div>
                  ))
              ) : (
                <span className="text-preview__empty">No preview available</span>
              )}
            </div>
          </div>
        </div>

        {preview && (
          <div className="text-preview__stats">
            <div className="text-preview__stat">
              <span className="text-preview__stat-label">Lines:</span>
              <span className="text-preview__stat-value">{preview.line_count}</span>
            </div>
            <div className="text-preview__stat">
              <span className="text-preview__stat-label">Est. Width:</span>
              <span className="text-preview__stat-value">{preview.estimated_width}px</span>
            </div>
            <div className="text-preview__stat">
              <span className="text-preview__stat-label">Est. Height:</span>
              <span className="text-preview__stat-value">{preview.estimated_height}px</span>
            </div>
            <div className="text-preview__stat">
              <span className="text-preview__stat-label">Characters:</span>
              <span className="text-preview__stat-value">{text.length}</span>
            </div>
          </div>
        )}

        <div className="text-preview__help">
          <p>
            <strong>Tip:</strong> SPO typically displays 28 characters per line with 3 lines max.
            Text longer than this will scroll or be truncated.
          </p>
        </div>
      </div>
    </div>
  );
}
