import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FrameAnnotation, FrameTag } from '../types/frameTags';
import { FrameTagger } from './FrameTagger';

interface AnnotationPanelProps {
  fighterId: string;
  fighterName: string;
  frameIndex: number;
  onAnnotationChange?: (annotation: FrameAnnotation) => void;
}

export const AnnotationPanel: React.FC<AnnotationPanelProps> = ({
  fighterId,
  fighterName,
  frameIndex,
  onAnnotationChange,
}) => {
  const [annotation, setAnnotation] = useState<FrameAnnotation | null>(null);
  const [tags, setTags] = useState<FrameTag[]>([]);
  const [loading, setLoading] = useState(false);
  const [notes, setNotes] = useState('');
  const [hitboxDescription, setHitboxDescription] = useState('');
  const [showTagger, setShowTagger] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load annotation and tags
  useEffect(() => {
    loadAnnotation();
    loadTags();
  }, [fighterId, frameIndex]);

  const loadAnnotation = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<FrameAnnotation | null>('get_frame_annotation', {
        fighterId,
        frameIndex,
      });
      
      setAnnotation(result);
      setNotes(result?.notes || '');
      setHitboxDescription(result?.hitbox_description || '');
    } catch (e) {
      console.error('Failed to load annotation:', e);
      setError('Failed to load annotation');
    } finally {
      setLoading(false);
    }
  };

  const loadTags = async () => {
    try {
      const result = await invoke<FrameTag[]>('get_frame_tags');
      setTags(result);
    } catch (e) {
      console.error('Failed to load tags:', e);
    }
  };

  const handleSaveAnnotation = useCallback(async () => {
    try {
      const updatedAnnotation: FrameAnnotation = {
        frame_index: frameIndex,
        tags: annotation?.tags || [],
        notes: notes.trim(),
        hitbox_description: hitboxDescription.trim() || undefined,
      };

      await invoke('update_frame_annotation', {
        fighterId,
        fighterName,
        frameIndex,
        annotation: updatedAnnotation,
      });

      setAnnotation(updatedAnnotation);
      onAnnotationChange?.(updatedAnnotation);
    } catch (e) {
      console.error('Failed to save annotation:', e);
      setError('Failed to save annotation');
    }
  }, [fighterId, fighterName, frameIndex, annotation, notes, hitboxDescription, onAnnotationChange]);

  const handleAddTag = async (tagId: string) => {
    try {
      const result = await invoke<FrameAnnotation>('add_tag_to_frame', {
        fighterId,
        fighterName,
        frameIndex,
        tagId,
      });
      
      setAnnotation(result);
      onAnnotationChange?.(result);
    } catch (e) {
      console.error('Failed to add tag:', e);
    }
  };

  const handleRemoveTag = async (tagId: string) => {
    try {
      const result = await invoke<FrameAnnotation>('remove_tag_from_frame', {
        fighterId,
        fighterName,
        frameIndex,
        tagId,
      });
      
      setAnnotation(result);
      onAnnotationChange?.(result);
    } catch (e) {
      console.error('Failed to remove tag:', e);
    }
  };

  const handleCreateTag = async (newTag: Omit<FrameTag, 'created_at' | 'created_by'>) => {
    try {
      const tag: FrameTag = {
        ...newTag,
        created_at: new Date().toISOString(),
        created_by: 'user',
      };
      
      await invoke('add_frame_tag', { tag });
      await loadTags();
      
      // Auto-select the new tag
      await handleAddTag(tag.id);
    } catch (e) {
      console.error('Failed to create tag:', e);
    }
  };

  // Auto-save notes after a delay
  useEffect(() => {
    const timeout = setTimeout(() => {
      if (annotation && (notes !== annotation.notes || hitboxDescription !== (annotation.hitbox_description || ''))) {
        handleSaveAnnotation();
      }
    }, 1000);

    return () => clearTimeout(timeout);
  }, [notes, hitboxDescription]);

  const appliedTagIds = annotation?.tags || [];

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>📝 Frame Annotation</h3>
        <span style={styles.frameInfo}>{fighterName} - Frame {frameIndex}</span>
      </div>

      {error && (
        <div style={styles.error}>{error}</div>
      )}

      {loading ? (
        <div style={styles.loading}>Loading...</div>
      ) : (
        <>
          {/* Tags Section */}
          <div style={styles.section}>
            <div style={styles.sectionHeader}>
              <span style={styles.sectionTitle}>Tags</span>
              <button
                onClick={() => setShowTagger(!showTagger)}
                style={styles.toggleButton}
              >
                {showTagger ? 'Hide' : 'Edit Tags'}
              </button>
            </div>

            {appliedTagIds.length > 0 ? (
              <div style={styles.appliedTags}>
                {appliedTagIds.map(tagId => {
                  const tag = tags.find(t => t.id === tagId);
                  if (!tag) return null;
                  return (
                    <span
                      key={tagId}
                      style={{
                        ...styles.tagBadge,
                        backgroundColor: tag.color,
                      }}
                      title={tag.description}
                    >
                      {tag.display_name}
                    </span>
                  );
                })}
              </div>
            ) : (
              <div style={styles.noTags}>No tags applied</div>
            )}

            {showTagger && (
              <div style={styles.taggerContainer}>
                <FrameTagger
                  tags={tags}
                  selectedTags={appliedTagIds}
                  onTagSelect={handleAddTag}
                  onTagRemove={handleRemoveTag}
                  onCreateTag={handleCreateTag}
                />
              </div>
            )}
          </div>

          {/* Notes Section */}
          <div style={styles.section}>
            <label style={styles.sectionTitle}>Notes</label>
            <textarea
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder="Add notes about this frame..."
              style={styles.notesInput}
              rows={4}
            />
          </div>

          {/* Hitbox Description Section */}
          <div style={styles.section}>
            <label style={styles.sectionTitle}>Hitbox Description</label>
            <textarea
              value={hitboxDescription}
              onChange={(e) => setHitboxDescription(e.target.value)}
              placeholder="Describe hitbox properties, damage, etc..."
              style={styles.notesInput}
              rows={2}
            />
          </div>
        </>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: 16,
    padding: 16,
    backgroundColor: '#16161e',
    borderRadius: 8,
    color: '#fff',
  },
  header: {
    display: 'flex',
    flexDirection: 'column',
    gap: 4,
    paddingBottom: 12,
    borderBottom: '1px solid #333',
  },
  title: {
    margin: 0,
    fontSize: 16,
    fontWeight: 600,
  },
  frameInfo: {
    fontSize: 12,
    color: '#888',
  },
  error: {
    padding: 8,
    backgroundColor: '#dc262620',
    border: '1px solid #dc2626',
    borderRadius: 4,
    color: '#f87171',
    fontSize: 12,
  },
  loading: {
    padding: 24,
    textAlign: 'center',
    color: '#888',
    fontSize: 13,
  },
  section: {
    display: 'flex',
    flexDirection: 'column',
    gap: 8,
  },
  sectionHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  sectionTitle: {
    fontSize: 12,
    fontWeight: 600,
    color: '#888',
    textTransform: 'uppercase',
    letterSpacing: 0.5,
  },
  toggleButton: {
    padding: '4px 10px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: 11,
    cursor: 'pointer',
  },
  appliedTags: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: 6,
  },
  tagBadge: {
    padding: '4px 10px',
    borderRadius: 12,
    fontSize: 11,
    fontWeight: 500,
    color: '#fff',
    textShadow: '0 1px 2px rgba(0,0,0,0.3)',
  },
  noTags: {
    fontSize: 12,
    color: '#666',
    fontStyle: 'italic',
  },
  taggerContainer: {
    marginTop: 8,
    maxHeight: 300,
    overflow: 'auto',
  },
  notesInput: {
    padding: 10,
    backgroundColor: '#1e1e2e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: 13,
    fontFamily: 'inherit',
    resize: 'vertical',
    lineHeight: 1.5,
  },
};
