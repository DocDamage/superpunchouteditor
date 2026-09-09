import React, { useState, useMemo } from 'react';
import { FrameTag, TagCategory, TAG_CATEGORIES, getCategoryInfo } from '../types/frameTags';

interface FrameTaggerProps {
  tags: FrameTag[];
  selectedTags: string[];
  onTagSelect: (tagId: string) => void;
  onTagRemove: (tagId: string) => void;
  onCreateTag?: (tag: Omit<FrameTag, 'created_at' | 'created_by'>) => void;
  readOnly?: boolean;
}

export const FrameTagger: React.FC<FrameTaggerProps> = ({
  tags,
  selectedTags,
  onTagSelect,
  onTagRemove,
  onCreateTag,
  readOnly = false,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<TagCategory | 'all'>('all');
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newTagName, setNewTagName] = useState('');
  const [newTagDisplayName, setNewTagDisplayName] = useState('');
  const [newTagCategory, setNewTagCategory] = useState<TagCategory>('misc');
  const [newTagDescription, setNewTagDescription] = useState('');

  // Filter tags based on search and category
  const filteredTags = useMemo(() => {
    return tags.filter(tag => {
      const matchesSearch = searchQuery === '' ||
        tag.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        tag.display_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        tag.description.toLowerCase().includes(searchQuery.toLowerCase());
      
      const matchesCategory = selectedCategory === 'all' || tag.category === selectedCategory;
      
      return matchesSearch && matchesCategory;
    });
  }, [tags, searchQuery, selectedCategory]);

  // Group tags by category
  const groupedTags = useMemo(() => {
    const groups: Record<TagCategory, FrameTag[]> = {
      idle: [],
      attack: [],
      defense: [],
      damage: [],
      knockdown: [],
      special: [],
      misc: [],
    };
    
    filteredTags.forEach(tag => {
      groups[tag.category].push(tag);
    });
    
    return groups;
  }, [filteredTags]);

  const handleCreateTag = () => {
    if (!newTagName.trim() || !newTagDisplayName.trim()) return;
    
    // Generate ID from name
    const id = newTagName.toLowerCase().replace(/\s+/g, '_').replace(/[^a-z0-9_]/g, '');
    
    onCreateTag?.({
      id,
      name: newTagName.toLowerCase().replace(/\s+/g, '_'),
      display_name: newTagDisplayName,
      description: newTagDescription,
      category: newTagCategory,
      color: getCategoryInfo(newTagCategory).color,
    });
    
    // Reset form
    setNewTagName('');
    setNewTagDisplayName('');
    setNewTagDescription('');
    setNewTagCategory('misc');
    setShowCreateForm(false);
  };

  return (
    <div style={styles.container}>
      {/* Search and Filter */}
      <div style={styles.header}>
        <input
          type="text"
          placeholder="Search tags..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          style={styles.searchInput}
        />
        
        <select
          value={selectedCategory}
          onChange={(e) => setSelectedCategory(e.target.value as TagCategory | 'all')}
          style={styles.categorySelect}
        >
          <option value="all">All Categories</option>
          {TAG_CATEGORIES.map(cat => (
            <option key={cat.value} value={cat.value}>
              {cat.icon} {cat.label}
            </option>
          ))}
        </select>
        
        {!readOnly && onCreateTag && (
          <button
            onClick={() => setShowCreateForm(!showCreateForm)}
            style={styles.createButton}
          >
            {showCreateForm ? 'Cancel' : '+ New Tag'}
          </button>
        )}
      </div>

      {/* Create Tag Form */}
      {showCreateForm && (
        <div style={styles.createForm}>
          <input
            type="text"
            placeholder="Tag name (e.g., custom_attack)"
            value={newTagName}
            onChange={(e) => setNewTagName(e.target.value)}
            style={styles.formInput}
          />
          <input
            type="text"
            placeholder="Display name (e.g., Custom Attack)"
            value={newTagDisplayName}
            onChange={(e) => setNewTagDisplayName(e.target.value)}
            style={styles.formInput}
          />
          <select
            value={newTagCategory}
            onChange={(e) => setNewTagCategory(e.target.value as TagCategory)}
            style={styles.formSelect}
          >
            {TAG_CATEGORIES.map(cat => (
              <option key={cat.value} value={cat.value}>
                {cat.icon} {cat.label}
              </option>
            ))}
          </select>
          <textarea
            placeholder="Description (optional)"
            value={newTagDescription}
            onChange={(e) => setNewTagDescription(e.target.value)}
            style={styles.formTextarea}
            rows={2}
          />
          <button onClick={handleCreateTag} style={styles.saveButton}>
            Create Tag
          </button>
        </div>
      )}

      {/* Selected Tags */}
      {selectedTags.length > 0 && (
        <div style={styles.selectedSection}>
          <div style={styles.sectionTitle}>Applied Tags</div>
          <div style={styles.tagList}>
            {selectedTags.map(tagId => {
              const tag = tags.find(t => t.id === tagId);
              if (!tag) return null;
              return (
                <span
                  key={tagId}
                  style={{
                    ...styles.tag,
                    backgroundColor: tag.color,
                  }}
                >
                  {getCategoryInfo(tag.category).icon} {tag.display_name}
                  {!readOnly && (
                    <button
                      onClick={() => onTagRemove(tagId)}
                      style={styles.removeButton}
                    >
                      ×
                    </button>
                  )}
                </span>
              );
            })}
          </div>
        </div>
      )}

      {/* Available Tags by Category */}
      <div style={styles.tagsSection}>
        {TAG_CATEGORIES.map(category => {
          const categoryTags = groupedTags[category.value];
          if (categoryTags.length === 0) return null;
          
          return (
            <div key={category.value} style={styles.categorySection}>
              <div style={styles.categoryHeader}>
                <span style={styles.categoryIcon}>{category.icon}</span>
                <span style={styles.categoryLabel}>{category.label}</span>
                <span style={styles.categoryCount}>({categoryTags.length})</span>
              </div>
              <div style={styles.tagList}>
                {categoryTags.map(tag => {
                  const isSelected = selectedTags.includes(tag.id);
                  return (
                    <button
                      key={tag.id}
                      onClick={() => !readOnly && (isSelected ? onTagRemove(tag.id) : onTagSelect(tag.id))}
                      style={{
                        ...styles.tag,
                        ...styles.clickableTag,
                        backgroundColor: isSelected ? tag.color : `${tag.color}40`,
                        opacity: isSelected ? 1 : 0.8,
                        cursor: readOnly ? 'default' : 'pointer',
                      }}
                      title={tag.description}
                      disabled={readOnly}
                    >
                      {isSelected && <span style={styles.checkmark}>✓ </span>}
                      {tag.display_name}
                    </button>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>

      {filteredTags.length === 0 && (
        <div style={styles.emptyState}>
          No tags found matching your search.
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: 12,
    padding: 12,
    backgroundColor: '#1e1e2e',
    borderRadius: 8,
    maxHeight: '100%',
    overflow: 'auto',
  },
  header: {
    display: 'flex',
    gap: 8,
    flexWrap: 'wrap',
  },
  searchInput: {
    flex: 1,
    minWidth: 150,
    padding: '8px 12px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: 13,
  },
  categorySelect: {
    padding: '8px 12px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: 13,
    minWidth: 120,
  },
  createButton: {
    padding: '8px 16px',
    backgroundColor: '#22c55e',
    border: 'none',
    borderRadius: 4,
    color: '#fff',
    fontSize: 13,
    cursor: 'pointer',
    fontWeight: 500,
  },
  createForm: {
    display: 'flex',
    flexDirection: 'column',
    gap: 8,
    padding: 12,
    backgroundColor: '#2a2a3e',
    borderRadius: 4,
    border: '1px solid #444',
  },
  formInput: {
    padding: '8px 12px',
    backgroundColor: '#1e1e2e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: 13,
  },
  formSelect: {
    padding: '8px 12px',
    backgroundColor: '#1e1e2e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: 13,
  },
  formTextarea: {
    padding: '8px 12px',
    backgroundColor: '#1e1e2e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: 13,
    resize: 'vertical',
    fontFamily: 'inherit',
  },
  saveButton: {
    padding: '8px 16px',
    backgroundColor: '#0066cc',
    border: 'none',
    borderRadius: 4,
    color: '#fff',
    fontSize: 13,
    cursor: 'pointer',
    fontWeight: 500,
  },
  selectedSection: {
    padding: 12,
    backgroundColor: '#2a2a3e',
    borderRadius: 4,
    border: '1px solid #444',
  },
  sectionTitle: {
    fontSize: 12,
    fontWeight: 600,
    color: '#888',
    textTransform: 'uppercase',
    marginBottom: 8,
    letterSpacing: 0.5,
  },
  tagsSection: {
    display: 'flex',
    flexDirection: 'column',
    gap: 16,
  },
  categorySection: {
    display: 'flex',
    flexDirection: 'column',
    gap: 8,
  },
  categoryHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
    fontSize: 13,
    fontWeight: 600,
    color: '#aaa',
  },
  categoryIcon: {
    fontSize: 14,
  },
  categoryLabel: {
    textTransform: 'capitalize',
  },
  categoryCount: {
    color: '#666',
    fontWeight: 400,
  },
  tagList: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: 6,
  },
  tag: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: 4,
    padding: '4px 10px',
    borderRadius: 12,
    fontSize: 12,
    fontWeight: 500,
    color: '#fff',
    textShadow: '0 1px 2px rgba(0,0,0,0.3)',
    border: '1px solid rgba(255,255,255,0.1)',
  },
  clickableTag: {
    transition: 'all 0.2s',
  },
  removeButton: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: 16,
    height: 16,
    padding: 0,
    marginLeft: 4,
    backgroundColor: 'rgba(0,0,0,0.3)',
    border: 'none',
    borderRadius: 8,
    color: '#fff',
    fontSize: 12,
    cursor: 'pointer',
    lineHeight: 1,
  },
  checkmark: {
    fontSize: 10,
  },
  emptyState: {
    padding: 24,
    textAlign: 'center',
    color: '#666',
    fontSize: 13,
  },
};
