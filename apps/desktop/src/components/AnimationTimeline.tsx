import React from 'react';
import { Animation, AnimationFrame } from './AnimationEditor';

interface AnimationTimelineProps {
  animation: Animation;
  selectedFrameIndex: number;
  onSelectFrame: (index: number) => void;
  onAddFrame: () => void;
  onRemoveFrame: (index: number) => void;
  onMoveFrame: (fromIndex: number, toIndex: number) => void;
  isEditing: boolean;
}

const frameWidth = 48;
const frameGap = 4;

export const AnimationTimeline: React.FC<AnimationTimelineProps> = ({
  animation,
  selectedFrameIndex,
  onSelectFrame,
  onAddFrame,
  onRemoveFrame,
  onMoveFrame,
  isEditing,
}) => {
  const totalDuration = animation.frames.reduce((sum, f) => sum + f.duration, 0);
  
  // Calculate frame positions for visual layout
  const getFrameStyle = (index: number, frame: AnimationFrame) => {
    const width = Math.max(frameWidth, (frame.duration / totalDuration) * 400);
    return {
      width: `${width}px`,
    };
  };

  const handleDragStart = (e: React.DragEvent, index: number) => {
    e.dataTransfer.setData('frameIndex', index.toString());
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  };

  const handleDrop = (e: React.DragEvent, targetIndex: number) => {
    e.preventDefault();
    const sourceIndex = parseInt(e.dataTransfer.getData('frameIndex'));
    if (sourceIndex !== targetIndex) {
      onMoveFrame(sourceIndex, targetIndex);
    }
  };

  return (
    <div className="h-full flex flex-col bg-slate-800 rounded-lg border border-slate-700 overflow-hidden">
      {/* Timeline Header */}
      <div className="flex items-center justify-between p-3 border-b border-slate-700 bg-slate-750">
        <div className="flex items-center gap-4">
          <span className="text-sm font-semibold text-slate-400">
            Timeline
          </span>
          <span className="text-xs text-slate-500">
            {animation.frames.length} frames • {totalDuration} frames total (~{(totalDuration / 60).toFixed(2)}s)
          </span>
          {animation.looping && (
            <span className="text-xs px-2 py-0.5 rounded bg-green-500/20 text-green-400">
              🔄 Looping
            </span>
          )}
        </div>
        {isEditing && (
          <button
            onClick={onAddFrame}
            className="px-3 py-1 text-xs bg-blue-600 hover:bg-blue-500 rounded font-medium"
          >
            + Add Frame
          </button>
        )}
      </div>

      {/* Timeline Ruler */}
      <div className="h-6 bg-slate-900 border-b border-slate-700 relative overflow-hidden">
        <div className="absolute inset-0 flex items-end px-2">
          {Array.from({ length: Math.ceil(totalDuration / 10) + 1 }).map((_, i) => (
            <div
              key={i}
              className="absolute bottom-0 border-l border-slate-600 text-xs text-slate-500 pl-1"
              style={{ left: `${i * 100}px` }}
            >
              {i * 10}f
            </div>
          ))}
        </div>
      </div>

      {/* Frame Strip */}
      <div className="flex-1 overflow-x-auto overflow-y-hidden p-4">
        <div className="flex items-center gap-1 min-w-max">
          {animation.frames.map((frame, index) => {
            const isSelected = index === selectedFrameIndex;
            const hasEffects = frame.effects.length > 0;
            
            return (
              <div
                key={index}
                draggable={isEditing}
                onDragStart={(e) => handleDragStart(e, index)}
                onDragOver={handleDragOver}
                onDrop={(e) => handleDrop(e, index)}
                onClick={() => onSelectFrame(index)}
                className={`
                  relative flex-shrink-0 h-24 rounded cursor-pointer transition-all
                  ${isSelected 
                    ? 'ring-2 ring-blue-500 ring-offset-2 ring-offset-slate-800' 
                    : 'hover:ring-1 hover:ring-slate-500'
                  }
                `}
                style={{
                  width: `${Math.max(48, frame.duration * 3)}px`,
                }}
              >
                {/* Frame Content */}
                <div 
                  className={`
                    h-full rounded flex flex-col items-center justify-center text-xs
                    ${isSelected 
                      ? 'bg-blue-600 text-white' 
                      : 'bg-slate-700 text-slate-400 hover:bg-slate-600'
                    }
                  `}
                >
                  <span className="font-semibold">Frame {index + 1}</span>
                  <span className="text-xs opacity-75">Pose {frame.pose_id}</span>
                  <span className="text-xs opacity-75">{frame.duration}f</span>
                  
                  {/* Effect indicators */}
                  {hasEffects && (
                    <div className="flex gap-0.5 mt-1">
                      {frame.effects.map((effect, ei) => (
                        <span 
                          key={ei}
                          className="text-[10px] px-1 rounded bg-black/30"
                          title={effect.type}
                        >
                          {effect.type === 'Shake' && '💥'}
                          {effect.type === 'Flash' && '⚡'}
                          {effect.type === 'Sound' && '🔊'}
                          {effect.type === 'Hitbox' && '🎯'}
                        </span>
                      ))}
                    </div>
                  )}
                </div>

                {/* Frame Number Indicator */}
                <div 
                  className={`
                    absolute -bottom-5 left-1/2 -translate-x-1/2 text-[10px]
                    ${isSelected ? 'text-blue-400 font-semibold' : 'text-slate-600'}
                  `}
                >
                  {index + 1}
                </div>

                {/* Remove button (edit mode only) */}
                {isEditing && animation.frames.length > 1 && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onRemoveFrame(index);
                    }}
                    className="absolute -top-2 -right-2 w-5 h-5 bg-red-500 hover:bg-red-400 rounded-full text-white text-xs flex items-center justify-center opacity-0 group-hover:opacity-100 transition"
                    style={{ opacity: isEditing ? 1 : 0 }}
                  >
                    ×
                  </button>
                )}
              </div>
            );
          })}

          {/* Loop indicator */}
          {animation.looping && animation.frames.length > 0 && (
            <div className="flex-shrink-0 flex items-center px-2">
              <div className="text-green-400 text-lg">↩️</div>
            </div>
          )}
        </div>
      </div>

      {/* Playback Controls */}
      <div className="p-3 border-t border-slate-700 bg-slate-750 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="text-xs text-slate-400">
            Frame {selectedFrameIndex + 1} / {animation.frames.length}
          </div>
          <div className="text-xs text-slate-500">
            Pose: {animation.frames[selectedFrameIndex]?.pose_id ?? '-'}
          </div>
          <div className="text-xs text-slate-500">
            Duration: {animation.frames[selectedFrameIndex]?.duration ?? '-'} frames
          </div>
        </div>

        {/* Frame Navigation */}
        <div className="flex items-center gap-1">
          <button
            onClick={() => onSelectFrame(Math.max(0, selectedFrameIndex - 1))}
            disabled={selectedFrameIndex === 0}
            className="px-3 py-1 text-xs bg-slate-700 hover:bg-slate-600 disabled:opacity-50 rounded"
          >
            ← Prev
          </button>
          <button
            onClick={() => onSelectFrame(Math.min(animation.frames.length - 1, selectedFrameIndex + 1))}
            disabled={selectedFrameIndex >= animation.frames.length - 1}
            className="px-3 py-1 text-xs bg-slate-700 hover:bg-slate-600 disabled:opacity-50 rounded"
          >
            Next →
          </button>
        </div>
      </div>
    </div>
  );
};
