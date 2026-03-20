import { useState } from 'react';
import { useStore, Color } from '../store/useStore';

export const PaletteEditor = () => {
  const { currentPalette, updateColor } = useStore();
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);

  if (!currentPalette) {
    return (
      <div className="empty-state">
        <p>No palette loaded for this boxer.</p>
      </div>
    );
  }

  const selectedColor = selectedIndex !== null ? currentPalette[selectedIndex] : null;

  const handleSliderChange = (channel: keyof Color, value: number) => {
    if (selectedIndex === null || !selectedColor) return;
    updateColor(selectedIndex, {
      ...selectedColor,
      [channel]: value,
    });
  };

  return (
    <div className="palette-editor">
      <h3 style={{ marginBottom: '1rem' }}>Palette Editor</h3>
      
      <div className="palette-grid">
        {currentPalette.map((color, idx) => (
          <div
            key={idx}
            className={`palette-swatch ${selectedIndex === idx ? 'selected' : ''}`}
            style={{ backgroundColor: `rgb(${color.r}, ${color.g}, ${color.b})` }}
            onClick={() => setSelectedIndex(idx)}
          />
        ))}
      </div>

      {selectedColor && (
        <div className="color-picker-panel">
          <div 
            className="color-preview-large" 
            style={{ backgroundColor: `rgb(${selectedColor.r}, ${selectedColor.g}, ${selectedColor.b})` }} 
          />
          
          <div className="slider-group">
            <div className="slider-row">
              <label style={{ color: '#ff4d4d' }}>R</label>
              <input 
                type="range" 
                min="0" 
                max="255" 
                step="8" // Snippet uses 5-bit color internally but displays 8-bit, 8 is a good step
                value={selectedColor.r} 
                onChange={(e) => handleSliderChange('r', parseInt(e.target.value))}
              />
              <span>{selectedColor.r}</span>
            </div>
            
            <div className="slider-row">
              <label style={{ color: '#2ecc71' }}>G</label>
              <input 
                type="range" 
                min="0" 
                max="255" 
                step="8"
                value={selectedColor.g} 
                onChange={(e) => handleSliderChange('g', parseInt(e.target.value))}
              />
              <span>{selectedColor.g}</span>
            </div>
            
            <div className="slider-row">
              <label style={{ color: '#3498db' }}>B</label>
              <input 
                type="range" 
                min="0" 
                max="255" 
                step="8"
                value={selectedColor.b} 
                onChange={(e) => handleSliderChange('b', parseInt(e.target.value))}
              />
              <span>{selectedColor.b}</span>
            </div>

            <div style={{ marginTop: '8px', color: 'var(--text-dim)', fontSize: '0.8rem' }}>
              Hex: #{selectedColor.r.toString(16).padStart(2, '0')}
              {selectedColor.g.toString(16).padStart(2, '0')}
              {selectedColor.b.toString(16).padStart(2, '0')}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
