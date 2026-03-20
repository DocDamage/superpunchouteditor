import React, { useEffect, useState, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';
import { AnimationTimeline } from './AnimationTimeline';
import { AnimationPreview } from './AnimationPreview';

// Types matching Rust structures
export interface FrameEffect {
  type: 'Shake' | 'Flash' | 'Sound' | 'Hitbox';
  data?: number | { x: number; y: number; w: number; h: number };
}

export interface AnimationFrame {
  pose_id: number;
  duration: number;
  tileset_id: number;
  effects: FrameEffect[];
}

export interface Animation {
  name: string;
  frames: AnimationFrame[];
  looping: boolean;
  category: AnimationCategory;
}

export type AnimationCategory = 
  | 'Idle' 
  | 'PunchLeft' 
  | 'PunchRight' 
  | 'Dodge' 
  | 'Hit' 
  | 'Knockdown' 
  | 'Special'
  | { Custom: string };

export interface FighterAnimations {
  fighter_id: number;
  fighter_name: string;
  animations: Animation[];
}

export interface AnimationCategoryInfo {
  value: string;
  label: string;
  icon: string;
  description: string;
}

const categoryColors: Record<string, string> = {
  Idle: '#22c55e',
  PunchLeft: '#f59e0b',
  PunchRight: '#ef4444',
  Dodge: '#3b82f6',
  Hit: '#8b5cf6',
  Knockdown: '#dc2626',
  Special: '#ec4899',
};

export const AnimationEditor: React.FC = () => {
  const { fighters, loadFighterList } = useStore();
  
  const [selectedFighterId, setSelectedFighterId] = useState<number | null>(null);
  const [fighterAnimations, setFighterAnimations] = useState<FighterAnimations | null>(null);
  const [categories, setCategories] = useState<AnimationCategoryInfo[]>([]);
  const [loading, setLoading] = useState(false);
  
  // Animation selection
  const [selectedAnimationIndex, setSelectedAnimationIndex] = useState<number>(0);
  const [selectedFrameIndex, setSelectedFrameIndex] = useState<number>(0);
  
  // Edit mode
  const [isEditing, setIsEditing] = useState(false);
  const [editedAnimation, setEditedAnimation] = useState<Animation | null>(null);
  const [showNewAnimDialog, setShowNewAnimDialog] = useState(false);
  const [newAnimName, setNewAnimName] = useState('');
  const [newAnimCategory, setNewAnimCategory] = useState<string>('Idle');
  
  // Validation
  const [validationResult, setValidationResult] = useState<{
    valid: boolean;
    warnings: string[];
    errors: string[];
    total_frames: number;
    total_duration: number;
    duration_seconds: number;
  } | null>(null);

  // Load fighters on mount
  useEffect(() => {
    loadFighterList();
    loadCategories();
  }, []);

  // Load animations when fighter changes
  useEffect(() => {
    if (selectedFighterId !== null) {
      loadAnimations(selectedFighterId);
    }
  }, [selectedFighterId]);

  // Update edited animation when selection changes
  useEffect(() => {
    if (fighterAnimations && fighterAnimations.animations.length > 0) {
      const anim = fighterAnimations.animations[selectedAnimationIndex];
      setEditedAnimation(JSON.parse(JSON.stringify(anim)));
    }
  }, [selectedAnimationIndex, fighterAnimations]);

  // Validate animation when edited
  useEffect(() => {
    if (editedAnimation) {
      validateCurrentAnimation();
    }
  }, [editedAnimation]);

  const loadCategories = async () => {
    try {
      const cats = await invoke<AnimationCategoryInfo[]>('get_animation_categories');
      setCategories(cats);
    } catch (e) {
      console.error('Failed to load categories:', e);
    }
  };

  const loadAnimations = async (fighterId: number) => {
    setLoading(true);
    try {
      const result = await invoke<FighterAnimations>('get_fighter_animations', { fighterId });
      setFighterAnimations(result);
      setSelectedAnimationIndex(0);
      setSelectedFrameIndex(0);
    } catch (e) {
      console.error('Failed to load animations:', e);
    } finally {
      setLoading(false);
    }
  };

  const validateCurrentAnimation = async () => {
    if (!editedAnimation) return;
    try {
      const result = await invoke<{
        valid: boolean;
        warnings: string[];
        errors: string[];
        total_frames: number;
        total_duration: number;
        duration_seconds: number;
      }>('validate_animation', { animation: editedAnimation });
      setValidationResult(result);
    } catch (e) {
      console.error('Validation failed:', e);
    }
  };

  const handleAddFrame = () => {
    if (!editedAnimation) return;
    
    const newFrame: AnimationFrame = {
      pose_id: 0,
      duration: 4,
      tileset_id: 0,
      effects: [],
    };
    
    setEditedAnimation({
      ...editedAnimation,
      frames: [...editedAnimation.frames, newFrame],
    });
    setSelectedFrameIndex(editedAnimation.frames.length);
  };

  const handleRemoveFrame = (index: number) => {
    if (!editedAnimation || editedAnimation.frames.length <= 1) return;
    
    const newFrames = [...editedAnimation.frames];
    newFrames.splice(index, 1);
    
    setEditedAnimation({
      ...editedAnimation,
      frames: newFrames,
    });
    
    if (selectedFrameIndex >= newFrames.length) {
      setSelectedFrameIndex(Math.max(0, newFrames.length - 1));
    }
  };

  const handleUpdateFrame = (index: number, updates: Partial<AnimationFrame>) => {
    if (!editedAnimation) return;
    
    const newFrames = [...editedAnimation.frames];
    newFrames[index] = { ...newFrames[index], ...updates };
    
    setEditedAnimation({
      ...editedAnimation,
      frames: newFrames,
    });
  };

  const handleMoveFrame = (fromIndex: number, toIndex: number) => {
    if (!editedAnimation) return;
    
    const newFrames = [...editedAnimation.frames];
    const [moved] = newFrames.splice(fromIndex, 1);
    newFrames.splice(toIndex, 0, moved);
    
    setEditedAnimation({
      ...editedAnimation,
      frames: newFrames,
    });
    setSelectedFrameIndex(toIndex);
  };

  const handleToggleEffect = (frameIndex: number, effectType: FrameEffect['type']) => {
    if (!editedAnimation) return;
    
    const frame = editedAnimation.frames[frameIndex];
    const hasEffect = frame.effects.some(e => e.type === effectType);
    
    let newEffects: FrameEffect[];
    if (hasEffect) {
      newEffects = frame.effects.filter(e => e.type !== effectType);
    } else {
      const newEffect: FrameEffect = effectType === 'Sound' 
        ? { type: 'Sound', data: 0 }
        : effectType === 'Hitbox'
        ? { type: 'Hitbox', data: { x: 0, y: 0, w: 32, h: 32 } }
        : { type: effectType };
      newEffects = [...frame.effects, newEffect];
    }
    
    handleUpdateFrame(frameIndex, { effects: newEffects });
  };

  const handleCreateAnimation = () => {
    if (!newAnimName.trim() || !fighterAnimations) return;
    
    const newAnim: Animation = {
      name: newAnimName.trim(),
      frames: [{ pose_id: 0, duration: 4, tileset_id: 0, effects: [] }],
      looping: false,
      category: newAnimCategory as AnimationCategory,
    };
    
    setFighterAnimations({
      ...fighterAnimations,
      animations: [...fighterAnimations.animations, newAnim],
    });
    
    setSelectedAnimationIndex(fighterAnimations.animations.length);
    setNewAnimName('');
    setShowNewAnimDialog(false);
    setIsEditing(true);
  };

  const handleDeleteAnimation = () => {
    if (!fighterAnimations || fighterAnimations.animations.length <= 1) return;
    
    const newAnims = [...fighterAnimations.animations];
    newAnims.splice(selectedAnimationIndex, 1);
    
    setFighterAnimations({
      ...fighterAnimations,
      animations: newAnims,
    });
    
    setSelectedAnimationIndex(Math.min(selectedAnimationIndex, newAnims.length - 1));
  };

  const handleExportAnimation = async () => {
    if (!editedAnimation) return;
    try {
      const json = await invoke<string>('export_animation_to_json', { animation: editedAnimation });
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${editedAnimation.name.replace(/\s+/g, '_')}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error('Export failed:', e);
    }
  };

  const currentAnimation = isEditing && editedAnimation 
    ? editedAnimation 
    : (fighterAnimations?.animations[selectedAnimationIndex] || null);

  const currentFrame = currentAnimation?.frames[selectedFrameIndex] || null;

  return (
    <div className="flex flex-col h-full bg-slate-900 text-white p-6">
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-2xl font-bold text-blue-400">Animation Editor</h1>
        <div className="flex items-center gap-4">
          <span className="text-sm text-slate-400">
            {selectedFighterId !== null && fighterAnimations 
              ? `Editing: ${fighterAnimations.boxer_name}` 
              : 'Select a fighter to edit animations'}
          </span>
        </div>
      </div>

      <div className="flex gap-4 h-full overflow-hidden">
        {/* Left Sidebar: Fighter & Animation List */}
        <div className="w-72 flex flex-col gap-4 overflow-hidden">
          {/* Fighter Selection */}
          <div className="bg-slate-800 rounded-lg border border-slate-700 overflow-hidden">
            <div className="p-3 border-b border-slate-700 bg-slate-750">
              <span className="text-sm font-semibold text-slate-400">Fighter</span>
            </div>
            <div className="max-h-48 overflow-y-auto">
              {fighters.map((f) => (
                <button
                  key={f.id}
                  onClick={() => setSelectedFighterId(f.id)}
                  className={`w-full p-2 text-left text-sm border-b border-slate-700/50 last:border-0 transition ${
                    selectedFighterId === f.id 
                      ? 'bg-blue-600 text-white' 
                      : 'hover:bg-slate-700 text-slate-300'
                  }`}
                >
                  {f.name}
                </button>
              ))}
            </div>
          </div>

          {/* Animation List */}
          {fighterAnimations && (
            <div className="flex-1 bg-slate-800 rounded-lg border border-slate-700 overflow-hidden flex flex-col">
              <div className="p-3 border-b border-slate-700 bg-slate-750 flex items-center justify-between">
                <span className="text-sm font-semibold text-slate-400">
                  Animations ({fighterAnimations.animations.length})
                </span>
                <button
                  onClick={() => setShowNewAnimDialog(true)}
                  className="px-2 py-1 text-xs bg-blue-600 hover:bg-blue-500 rounded"
                >
                  + New
                </button>
              </div>
              <div className="flex-1 overflow-y-auto">
                {fighterAnimations.animations.map((anim, idx) => (
                  <button
                    key={idx}
                    onClick={() => {
                      setSelectedAnimationIndex(idx);
                      setIsEditing(false);
                    }}
                    className={`w-full p-3 text-left border-b border-slate-700/50 transition ${
                      selectedAnimationIndex === idx && !isEditing
                        ? 'bg-slate-700' 
                        : 'hover:bg-slate-700/50'
                    }`}
                  >
                    <div className="flex items-center gap-2">
                      <span 
                        className="text-xs px-2 py-0.5 rounded"
                        style={{ 
                          backgroundColor: `${categoryColors[anim.category as string] || '#6b7280'}20`,
                          color: categoryColors[anim.category as string] || '#6b7280'
                        }}
                      >
                        {anim.category}
                      </span>
                      {anim.looping && (
                        <span className="text-xs text-slate-500">🔄</span>
                      )}
                    </div>
                    <div className="text-sm text-slate-300 mt-1">{anim.name}</div>
                    <div className="text-xs text-slate-500">
                      {anim.frames.length} frames • {(anim.frames.reduce((a, f) => a + f.duration, 0) / 60).toFixed(2)}s
                    </div>
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Main Content Area */}
        <div className="flex-1 flex flex-col gap-4 overflow-hidden">
          {loading ? (
            <div className="flex-1 flex items-center justify-center">
              <div className="animate-spin rounded-full h-12 w-12 border-4 border-blue-500 border-t-transparent"></div>
            </div>
          ) : currentAnimation ? (
            <>
              {/* Animation Header */}
              <div className="bg-slate-800 rounded-lg p-4 border border-slate-700">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-3">
                    {isEditing ? (
                      <input
                        type="text"
                        value={editedAnimation?.name || ''}
                        onChange={(e) => editedAnimation && setEditedAnimation({ ...editedAnimation, name: e.target.value })}
                        className="bg-slate-900 border border-slate-600 rounded px-3 py-1 text-lg font-semibold"
                      />
                    ) : (
                      <h2 className="text-lg font-semibold">{currentAnimation.name}</h2>
                    )}
                    <span 
                      className="text-xs px-2 py-1 rounded"
                      style={{ 
                        backgroundColor: `${categoryColors[currentAnimation.category as string] || '#6b7280'}20`,
                        color: categoryColors[currentAnimation.category as string] || '#6b7280'
                      }}
                    >
                      {currentAnimation.category}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => isEditing ? setIsEditing(false) : setIsEditing(true)}
                      className={`px-3 py-1.5 rounded text-sm font-medium transition ${
                        isEditing
                          ? 'bg-slate-600 hover:bg-slate-500'
                          : 'bg-blue-600 hover:bg-blue-500'
                      }`}
                    >
                      {isEditing ? 'Cancel' : 'Edit'}
                    </button>
                    {isEditing && (
                      <button
                        onClick={() => setIsEditing(false)}
                        className="px-3 py-1.5 rounded text-sm font-medium bg-green-600 hover:bg-green-500"
                      >
                        Save
                      </button>
                    )}
                    <button
                      onClick={handleExportAnimation}
                      className="px-3 py-1.5 rounded text-sm font-medium bg-slate-700 hover:bg-slate-600"
                      title="Export to JSON"
                    >
                      Export
                    </button>
                    <button
                      onClick={handleDeleteAnimation}
                      disabled={fighterAnimations?.animations.length === 1}
                      className="px-3 py-1.5 rounded text-sm font-medium bg-red-600 hover:bg-red-500 disabled:opacity-50"
                    >
                      Delete
                    </button>
                  </div>
                </div>

                {/* Animation Controls */}
                {isEditing && editedAnimation && (
                  <div className="flex items-center gap-4 pt-3 border-t border-slate-700">
                    <label className="flex items-center gap-2">
                      <input
                        type="checkbox"
                        checked={editedAnimation.looping}
                        onChange={(e) => setEditedAnimation({ ...editedAnimation, looping: e.target.checked })}
                        className="rounded"
                      />
                      <span className="text-sm text-slate-400">Loop</span>
                    </label>
                    <label className="flex items-center gap-2">
                      <span className="text-sm text-slate-400">Category:</span>
                      <select
                        value={typeof editedAnimation.category === 'string' ? editedAnimation.category : 'Idle'}
                        onChange={(e) => setEditedAnimation({ ...editedAnimation, category: e.target.value as AnimationCategory })}
                        className="bg-slate-900 border border-slate-600 rounded px-2 py-1 text-sm"
                      >
                        {categories.map(cat => (
                          <option key={cat.value} value={cat.value}>
                            {cat.icon} {cat.label}
                          </option>
                        ))}
                      </select>
                    </label>
                  </div>
                )}
              </div>

              {/* Animation Preview */}
              <div className="flex-shrink-0 h-64">
                <AnimationPreview
                  fighterId={selectedFighterId}
                  currentFrame={currentFrame}
                  isPlaying={true}
                />
              </div>

              {/* Timeline */}
              <div className="flex-1 min-h-0">
                <AnimationTimeline
                  animation={currentAnimation}
                  selectedFrameIndex={selectedFrameIndex}
                  onSelectFrame={setSelectedFrameIndex}
                  onAddFrame={handleAddFrame}
                  onRemoveFrame={handleRemoveFrame}
                  onMoveFrame={handleMoveFrame}
                  isEditing={isEditing}
                />
              </div>

              {/* Frame Editor */}
              {currentFrame && isEditing && (
                <div className="bg-slate-800 rounded-lg p-4 border border-slate-700">
                  <h3 className="text-sm font-semibold text-slate-400 uppercase mb-3">
                    Frame {selectedFrameIndex + 1} of {currentAnimation.frames.length}
                  </h3>
                  <div className="grid grid-cols-4 gap-4">
                    <div>
                      <label className="block text-xs text-slate-500 mb-1">Pose ID</label>
                      <input
                        type="number"
                        min={0}
                        max={127}
                        value={currentFrame.pose_id}
                        onChange={(e) => handleUpdateFrame(selectedFrameIndex, { pose_id: parseInt(e.target.value) || 0 })}
                        className="w-full bg-slate-900 border border-slate-600 rounded px-3 py-2"
                      />
                    </div>
                    <div>
                      <label className="block text-xs text-slate-500 mb-1">Duration (frames)</label>
                      <input
                        type="number"
                        min={1}
                        max={255}
                        value={currentFrame.duration}
                        onChange={(e) => handleUpdateFrame(selectedFrameIndex, { duration: parseInt(e.target.value) || 1 })}
                        className="w-full bg-slate-900 border border-slate-600 rounded px-3 py-2"
                      />
                    </div>
                    <div>
                      <label className="block text-xs text-slate-500 mb-1">Tileset ID</label>
                      <input
                        type="number"
                        min={0}
                        max={255}
                        value={currentFrame.tileset_id}
                        onChange={(e) => handleUpdateFrame(selectedFrameIndex, { tileset_id: parseInt(e.target.value) || 0 })}
                        className="w-full bg-slate-900 border border-slate-600 rounded px-3 py-2"
                      />
                    </div>
                    <div>
                      <label className="block text-xs text-slate-500 mb-1">Effects</label>
                      <div className="flex gap-2">
                        {(['Shake', 'Flash', 'Sound', 'Hitbox'] as const).map(effect => (
                          <button
                            key={effect}
                            onClick={() => handleToggleEffect(selectedFrameIndex, effect)}
                            className={`px-2 py-1 rounded text-xs font-medium transition ${
                              currentFrame.effects.some(e => e.type === effect)
                                ? 'bg-blue-600 text-white'
                                : 'bg-slate-700 text-slate-400 hover:bg-slate-600'
                            }`}
                          >
                            {effect}
                          </button>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Validation Results */}
              {validationResult && (
                <div className={`rounded-lg p-3 border ${
                  validationResult.errors.length > 0
                    ? 'bg-red-500/10 border-red-500/30'
                    : validationResult.warnings.length > 0
                    ? 'bg-amber-500/10 border-amber-500/30'
                    : 'bg-green-500/10 border-green-500/30'
                }`}>
                  <div className="flex items-center gap-4 text-sm">
                    <span className="text-slate-400">
                      {validationResult.total_frames} frames • {validationResult.duration_seconds.toFixed(2)}s
                    </span>
                    {validationResult.errors.length > 0 && (
                      <span className="text-red-400">
                        {validationResult.errors.length} error(s)
                      </span>
                    )}
                    {validationResult.warnings.length > 0 && (
                      <span className="text-amber-400">
                        {validationResult.warnings.length} warning(s)
                      </span>
                    )}
                  </div>
                  {validationResult.errors.map((err, i) => (
                    <div key={`err-${i}`} className="text-red-400 text-xs mt-1">• {err}</div>
                  ))}
                  {validationResult.warnings.map((warn, i) => (
                    <div key={`warn-${i}`} className="text-amber-400 text-xs mt-1">• {warn}</div>
                  ))}
                </div>
              )}
            </>
          ) : (
            <div className="flex-1 flex items-center justify-center text-slate-500">
              <div className="text-center">
                <div className="text-4xl mb-4">🎬</div>
                <p>Select a fighter to view and edit animations</p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* New Animation Dialog */}
      {showNewAnimDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-slate-800 rounded-lg p-6 border border-slate-700 w-96">
            <h3 className="text-lg font-semibold mb-4">New Animation</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-slate-400 mb-1">Name</label>
                <input
                  type="text"
                  value={newAnimName}
                  onChange={(e) => setNewAnimName(e.target.value)}
                  placeholder="e.g., Uppercut"
                  className="w-full bg-slate-900 border border-slate-600 rounded px-3 py-2"
                  autoFocus
                />
              </div>
              <div>
                <label className="block text-sm text-slate-400 mb-1">Category</label>
                <select
                  value={newAnimCategory}
                  onChange={(e) => setNewAnimCategory(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-600 rounded px-3 py-2"
                >
                  {categories.map(cat => (
                    <option key={cat.value} value={cat.value}>
                      {cat.icon} {cat.label}
                    </option>
                  ))}
                </select>
              </div>
            </div>
            <div className="flex gap-2 mt-6">
              <button
                onClick={() => setShowNewAnimDialog(false)}
                className="flex-1 px-4 py-2 rounded bg-slate-700 hover:bg-slate-600"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateAnimation}
                disabled={!newAnimName.trim()}
                className="flex-1 px-4 py-2 rounded bg-blue-600 hover:bg-blue-500 disabled:opacity-50"
              >
                Create
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
