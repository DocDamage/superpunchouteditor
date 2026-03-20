import { useState, useCallback, useEffect } from 'react';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useStore, ProjectThumbnail } from '../store/useStore';
import { ProjectThumbnailDisplay, ThumbnailCaptureButton, ThumbnailManager } from './ProjectThumbnail';

interface NewProjectDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (name: string, author: string, description: string, path: string) => void;
}

function NewProjectDialog({ isOpen, onClose, onCreate }: NewProjectDialogProps) {
  const [name, setName] = useState('');
  const [author, setAuthor] = useState('');
  const [description, setDescription] = useState('');
  const [error, setError] = useState<string | null>(null);

  const handleSelectLocation = async () => {
    if (!name.trim()) {
      setError('Please enter a project name');
      return;
    }

    const selected = await save({
      defaultPath: `${name.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.spo`,
      filters: [{
        name: 'SPO Project',
        extensions: ['spo']
      }]
    });

    if (selected) {
      onCreate(name, author, description, selected);
      // Reset form
      setName('');
      setAuthor('');
      setDescription('');
      setError(null);
    }
  };

  const handleCancel = () => {
    setName('');
    setAuthor('');
    setDescription('');
    setError(null);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.7)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000,
    }}>
      <div style={{
        backgroundColor: 'var(--panel-bg)',
        borderRadius: '12px',
        border: '1px solid var(--border)',
        padding: '2rem',
        width: '100%',
        maxWidth: '480px',
      }}>
        <h2 style={{ marginTop: 0, marginBottom: '1.5rem' }}>New Project</h2>
        
        {error && (
          <div style={{ 
            color: 'var(--accent)', 
            backgroundColor: 'rgba(255, 50, 50, 0.1)',
            padding: '0.75rem',
            borderRadius: '6px',
            marginBottom: '1rem',
            fontSize: '0.9rem'
          }}>
            {error}
          </div>
        )}

        <div style={{ marginBottom: '1rem' }}>
          <label style={{ display: 'block', marginBottom: '0.5rem', fontSize: '0.9rem', color: 'var(--text-dim)' }}>
            Project Name *
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g., My SPO Hack"
            style={{
              width: '100%',
              padding: '0.75rem',
              borderRadius: '6px',
              border: '1px solid var(--border)',
              backgroundColor: 'var(--glass)',
              color: 'var(--text)',
              fontSize: '1rem',
              boxSizing: 'border-box',
            }}
            autoFocus
          />
        </div>

        <div style={{ marginBottom: '1rem' }}>
          <label style={{ display: 'block', marginBottom: '0.5rem', fontSize: '0.9rem', color: 'var(--text-dim)' }}>
            Author
          </label>
          <input
            type="text"
            value={author}
            onChange={(e) => setAuthor(e.target.value)}
            placeholder="Your name"
            style={{
              width: '100%',
              padding: '0.75rem',
              borderRadius: '6px',
              border: '1px solid var(--border)',
              backgroundColor: 'var(--glass)',
              color: 'var(--text)',
              fontSize: '1rem',
              boxSizing: 'border-box',
            }}
          />
        </div>

        <div style={{ marginBottom: '1.5rem' }}>
          <label style={{ display: 'block', marginBottom: '0.5rem', fontSize: '0.9rem', color: 'var(--text-dim)' }}>
            Description
          </label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Brief description of your project..."
            rows={3}
            style={{
              width: '100%',
              padding: '0.75rem',
              borderRadius: '6px',
              border: '1px solid var(--border)',
              backgroundColor: 'var(--glass)',
              color: 'var(--text)',
              fontSize: '1rem',
              resize: 'vertical',
              boxSizing: 'border-box',
              fontFamily: 'inherit',
            }}
          />
        </div>

        <div style={{ display: 'flex', gap: '0.75rem', justifyContent: 'flex-end' }}>
          <button
            onClick={handleCancel}
            style={{
              padding: '0.75rem 1.5rem',
              borderRadius: '6px',
              border: '1px solid var(--border)',
              backgroundColor: 'transparent',
              color: 'var(--text)',
              cursor: 'pointer',
              fontSize: '0.95rem',
            }}
          >
            Cancel
          </button>
          <button
            onClick={handleSelectLocation}
            style={{
              padding: '0.75rem 1.5rem',
              borderRadius: '6px',
              border: 'none',
              backgroundColor: 'var(--blue)',
              color: 'white',
              cursor: 'pointer',
              fontSize: '0.95rem',
              fontWeight: 500,
            }}
          >
            Choose Location
          </button>
        </div>
      </div>
    </div>
  );
}

interface ProjectInfoDisplayProps {
  project: import('../store/useStore').ProjectFile;
  projectPath: string | null;
  isModified: boolean;
}

function ProjectInfoDisplay({ project, projectPath, isModified }: ProjectInfoDisplayProps) {
  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  };

  return (
    <div style={{
      backgroundColor: 'var(--glass)',
      borderRadius: '8px',
      padding: '1rem',
      marginBottom: '1rem',
    }}>
      <div style={{ 
        display: 'flex', 
        justifyContent: 'space-between', 
        alignItems: 'flex-start',
        marginBottom: '0.5rem'
      }}>
        <h3 style={{ margin: 0, fontSize: '1.1rem' }}>
          {project.metadata.name}
          {isModified && <span style={{ color: 'var(--accent)', marginLeft: '0.5rem' }}>*</span>}
        </h3>
        <span style={{ 
          fontSize: '0.75rem', 
          color: 'var(--text-dim)',
          backgroundColor: 'var(--panel-bg)',
          padding: '0.25rem 0.5rem',
          borderRadius: '4px',
        }}>
          v{project.metadata.version}
        </span>
      </div>
      
      {project.metadata.author && (
        <div style={{ fontSize: '0.85rem', color: 'var(--text-dim)', marginBottom: '0.25rem' }}>
          by {project.metadata.author}
        </div>
      )}
      
      {project.metadata.description && (
        <div style={{ 
          fontSize: '0.85rem', 
          color: 'var(--text)', 
          marginTop: '0.5rem',
          fontStyle: 'italic'
        }}>
          {project.metadata.description}
        </div>
      )}
      
      <div style={{ 
        display: 'grid', 
        gridTemplateColumns: '1fr 1fr', 
        gap: '0.5rem',
        marginTop: '0.75rem',
        fontSize: '0.75rem',
        color: 'var(--text-dim)',
      }}>
        <div>Created: {formatDate(project.metadata.created_at)}</div>
        <div>Modified: {formatDate(project.metadata.modified_at)}</div>
      </div>
      
      {projectPath && (
        <div style={{ 
          marginTop: '0.5rem',
          fontSize: '0.75rem',
          color: 'var(--text-dim)',
          wordBreak: 'break-all'
        }}>
          {projectPath}
        </div>
      )}

      <div style={{
        marginTop: '0.75rem',
        paddingTop: '0.75rem',
        borderTop: '1px solid var(--border)',
        fontSize: '0.8rem',
        display: 'flex',
        gap: '1rem',
      }}>
        <span>Edits: {project.edits.length}</span>
        <span>Assets: {project.assets.length}</span>
        <span>ROM SHA1: {project.rom_base_sha1.substring(0, 8)}...</span>
      </div>
    </div>
  );
}

export function ProjectManager() {
  const {
    currentProject,
    currentProjectPath,
    isProjectModified,
    romSha1,
    createProject,
    saveProject,
    loadProject,
    setError,
    setProjectModified,
  } = useStore();

  const [isNewProjectDialogOpen, setIsNewProjectDialogOpen] = useState(false);
  const [recentProjects, setRecentProjects] = useState<Array<{ 
    path: string; 
    name: string; 
    lastOpened: string;
    thumbnail?: ProjectThumbnail;
  }>>([]);
  const [recentThumbnailsLoaded, setRecentThumbnailsLoaded] = useState(false);

  const { loadThumbnailFromPath } = useStore();

  // Load recent projects from local storage
  useEffect(() => {
    const stored = localStorage.getItem('spo_recent_projects');
    if (stored) {
      try {
        const parsed = JSON.parse(stored);
        setRecentProjects(parsed);
      } catch (e) {
        console.error('Failed to parse recent projects:', e);
      }
    }
  }, []);

  // Load thumbnails for recent projects
  useEffect(() => {
    if (recentProjects.length === 0 || recentThumbnailsLoaded) return;

    const loadThumbnails = async () => {
      const updated = await Promise.all(
        recentProjects.map(async (project) => {
          try {
            const thumbnail = await loadThumbnailFromPath(project.path);
            return { ...project, thumbnail: thumbnail || undefined };
          } catch (e) {
            return project;
          }
        })
      );
      setRecentProjects(updated);
      setRecentThumbnailsLoaded(true);
    };

    loadThumbnails();
  }, [recentProjects, recentThumbnailsLoaded, loadThumbnailFromPath]);

  // Save recent projects when current project changes
  useEffect(() => {
    if (currentProject && currentProjectPath) {
      const newEntry = {
        path: currentProjectPath,
        name: currentProject.metadata.name,
        lastOpened: new Date().toISOString(),
      };
      
      const updated = [
        newEntry,
        ...recentProjects.filter(p => p.path !== currentProjectPath)
      ].slice(0, 10);
      
      setRecentProjects(updated);
      localStorage.setItem('spo_recent_projects', JSON.stringify(updated));
    }
  }, [currentProject, currentProjectPath]);

  const handleNewProject = useCallback(async () => {
    if (!romSha1) {
      setError('Please load a ROM first');
      return;
    }
    setIsNewProjectDialogOpen(true);
  }, [romSha1, setError]);

  const handleCreateProject = useCallback(async (
    name: string,
    author: string,
    description: string,
    path: string
  ) => {
    try {
      // Ensure path ends with .spo
      let projectPath = path;
      if (!path.endsWith('.spo')) {
        projectPath = path + '.spo';
      }
      
      await createProject(projectPath, name, author || undefined, description || undefined);
      setIsNewProjectDialogOpen(false);
    } catch (e) {
      console.error('Failed to create project:', e);
      setError((e as Error).toString());
    }
  }, [createProject, setError]);

  const handleSaveProject = useCallback(async () => {
    if (!currentProject) {
      // No project open, treat as save as
      const selected = await save({
        filters: [{
          name: 'SPO Project',
          extensions: ['spo']
        }]
      });
      
      if (selected) {
        try {
          await saveProject(selected);
        } catch (e) {
          console.error('Failed to save project:', e);
          setError((e as Error).toString());
        }
      }
      return;
    }

    try {
      await saveProject();
      setProjectModified(false);
    } catch (e) {
      console.error('Failed to save project:', e);
      setError((e as Error).toString());
    }
  }, [currentProject, saveProject, setError, setProjectModified]);

  const handleSaveWithThumbnail = useCallback(async () => {
    // First capture thumbnail, then save
    const { captureThumbnail, saveThumbnail } = useStore.getState();
    
    try {
      const thumbnail = await captureThumbnail('editor');
      if (thumbnail) {
        await saveThumbnail(thumbnail);
      }
    } catch (e) {
      console.error('Failed to capture thumbnail:', e);
      // Continue with save even if thumbnail fails
    }

    // Now save the project
    await handleSaveProject();
  }, [handleSaveProject]);

  const handleSaveProjectAs = useCallback(async () => {
    const selected = await save({
      defaultPath: currentProject?.metadata.name.replace(/[^a-z0-9]/gi, '_').toLowerCase() + '.spo',
      filters: [{
        name: 'SPO Project',
        extensions: ['spo']
      }]
    });
    
    if (selected) {
      try {
        await saveProject(selected);
      } catch (e) {
        console.error('Failed to save project:', e);
        setError((e as Error).toString());
      }
    }
  }, [currentProject, saveProject, setError]);

  const handleOpenProject = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'SPO Project',
        extensions: ['spo']
      }]
    });
    
    if (typeof selected === 'string') {
      try {
        await loadProject(selected);
      } catch (e) {
        console.error('Failed to load project:', e);
        setError((e as Error).toString());
      }
    }
  }, [loadProject, setError]);

  const handleOpenRecentProject = useCallback(async (path: string) => {
    try {
      await loadProject(path);
    } catch (e) {
      console.error('Failed to load recent project:', e);
      setError((e as Error).toString());
    }
  }, [loadProject, setError]);

  const handleCloseProject = useCallback(() => {
    // Just clear the current project from state
    useStore.setState({ 
      currentProject: null, 
      currentProjectPath: null,
      isProjectModified: false 
    });
  }, []);

  return (
    <div>
      {/* Project Menu Buttons */}
      <div style={{ 
        display: 'flex', 
        gap: '0.5rem', 
        marginBottom: '1rem',
        flexWrap: 'wrap'
      }}>
        <button
          onClick={handleNewProject}
          disabled={!romSha1}
          title={!romSha1 ? 'Load a ROM first' : undefined}
          style={{
            padding: '0.5rem 1rem',
            borderRadius: '6px',
            border: '1px solid var(--border)',
            backgroundColor: 'var(--glass)',
            color: 'var(--text)',
            cursor: romSha1 ? 'pointer' : 'not-allowed',
            opacity: romSha1 ? 1 : 0.5,
            fontSize: '0.85rem',
          }}
        >
          New Project
        </button>
        
        <button
          onClick={handleOpenProject}
          style={{
            padding: '0.5rem 1rem',
            borderRadius: '6px',
            border: '1px solid var(--border)',
            backgroundColor: 'var(--glass)',
            color: 'var(--text)',
            cursor: 'pointer',
            fontSize: '0.85rem',
          }}
        >
          Open Project
        </button>
        
        <button
          onClick={handleSaveProject}
          disabled={!romSha1}
          style={{
            padding: '0.5rem 1rem',
            borderRadius: '6px',
            border: '1px solid var(--border)',
            backgroundColor: currentProject ? 'var(--blue)' : 'var(--glass)',
            color: 'white',
            cursor: romSha1 ? 'pointer' : 'not-allowed',
            opacity: romSha1 ? 1 : 0.5,
            fontSize: '0.85rem',
          }}
        >
          {currentProject ? 'Save' : 'Save Project'}
        </button>
        
        {currentProject && (
          <button
            onClick={handleSaveProjectAs}
            style={{
              padding: '0.5rem 1rem',
              borderRadius: '6px',
              border: '1px solid var(--border)',
              backgroundColor: 'var(--glass)',
              color: 'var(--text)',
              cursor: 'pointer',
              fontSize: '0.85rem',
            }}
          >
            Save As...
          </button>
        )}
        
        {currentProject && (
          <button
            onClick={handleCloseProject}
            style={{
              padding: '0.5rem 1rem',
              borderRadius: '6px',
              border: '1px solid var(--border)',
              backgroundColor: 'var(--glass)',
              color: 'var(--text)',
              cursor: 'pointer',
              fontSize: '0.85rem',
            }}
          >
            Close
          </button>
        )}
      </div>

      {/* Current Project Info */}
      {currentProject ? (
        <>
          <ProjectInfoDisplay 
            project={currentProject} 
            projectPath={currentProjectPath}
            isModified={isProjectModified}
          />
          <ThumbnailManager style={{ marginBottom: '1rem' }} />
        </>
      ) : (
        <div style={{
          backgroundColor: 'var(--glass)',
          borderRadius: '8px',
          padding: '1rem',
          marginBottom: '1rem',
          textAlign: 'center',
          color: 'var(--text-dim)',
        }}>
          No project open. Create a new project or open an existing one.
        </div>
      )}

      {/* Recent Projects */}
      {recentProjects.length > 0 && !currentProject && (
        <div style={{ marginTop: '1rem' }}>
          <h4 style={{ 
            fontSize: '0.9rem', 
            color: 'var(--text-dim)',
            marginBottom: '0.5rem',
            textTransform: 'uppercase',
            letterSpacing: '0.05em',
          }}>
            Recent Projects
          </h4>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            {recentProjects.map((project, index) => (
              <button
                key={index}
                onClick={() => handleOpenRecentProject(project.path)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '1rem',
                  padding: '0.75rem',
                  borderRadius: '8px',
                  border: '1px solid var(--border)',
                  backgroundColor: 'var(--glass)',
                  color: 'var(--text)',
                  cursor: 'pointer',
                  textAlign: 'left',
                  width: '100%',
                }}
              >
                <ProjectThumbnailDisplay 
                  thumbnail={project.thumbnail || null} 
                  size="small"
                  showMeta={false}
                />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ 
                    fontWeight: 500, 
                    marginBottom: '0.25rem',
                    whiteSpace: 'nowrap',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                  }}>
                    {project.name}
                  </div>
                  <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>
                    {new Date(project.lastOpened).toLocaleDateString()}
                  </div>
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* New Project Dialog */}
      <NewProjectDialog
        isOpen={isNewProjectDialogOpen}
        onClose={() => setIsNewProjectDialogOpen(false)}
        onCreate={handleCreateProject}
      />
    </div>
  );
}
