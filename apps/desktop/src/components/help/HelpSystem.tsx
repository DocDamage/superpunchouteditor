import React, { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { createPortal } from 'react-dom';
import './HelpSystem.css';

interface HelpArticle {
  id: string;
  title: string;
  category: 'getting_started' | 'editing' | 'advanced' | 'troubleshooting';
  content: string;
  html_content?: string;
  tags: string[];
}

interface HelpArticleSummary {
  id: string;
  title: string;
  category: 'getting_started' | 'editing' | 'advanced' | 'troubleshooting';
  summary: string;
  tags: string[];
}

interface SearchResult {
  id: string;
  title: string;
  category: 'getting_started' | 'editing' | 'advanced' | 'troubleshooting';
  summary: string;
  tags: string[];
  relevance: number;
  matched_terms: string[];
}

type HelpCategory = 'getting_started' | 'editing' | 'advanced' | 'troubleshooting';

const categoryConfig: Record<HelpCategory, { label: string; icon: string; color: string }> = {
  getting_started: { label: 'Getting Started', icon: '🚀', color: '#4ade80' },
  editing: { label: 'Editing', icon: '✏️', color: '#60a5fa' },
  advanced: { label: 'Advanced', icon: '⚙️', color: '#f472b6' },
  troubleshooting: { label: 'Troubleshooting', icon: '🔧', color: '#fbbf24' },
};

interface HelpSystemProps {
  isOpen: boolean;
  onClose: () => void;
  initialArticleId?: string;
  initialContext?: string;
}

export const HelpSystem: React.FC<HelpSystemProps> = ({
  isOpen,
  onClose,
  initialArticleId,
  initialContext,
}) => {
  const [articles, setArticles] = useState<HelpArticleSummary[]>([]);
  const [currentArticle, setCurrentArticle] = useState<HelpArticle | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<HelpCategory | null>(null);
  const [history, setHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [loading, setLoading] = useState(false);
  const [feedbackSubmitted, setFeedbackSubmitted] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Load articles on mount
  useEffect(() => {
    if (isOpen) {
      loadArticles();
    }
  }, [isOpen]);

  // Handle initial article or context
  useEffect(() => {
    if (isOpen && articles.length > 0) {
      if (initialArticleId) {
        loadArticle(initialArticleId);
      } else if (initialContext) {
        loadContextHelp(initialContext);
      }
    }
  }, [isOpen, articles, initialArticleId, initialContext]);

  // Focus search input when opened
  useEffect(() => {
    if (isOpen && searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [isOpen]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return;

      if (e.key === 'Escape') {
        onClose();
      } else if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
        e.preventDefault();
        searchInputRef.current?.focus();
      } else if ((e.metaKey || e.ctrlKey) && e.key === '[') {
        e.preventDefault();
        goBack();
      } else if ((e.metaKey || e.ctrlKey) && e.key === ']') {
        e.preventDefault();
        goForward();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, historyIndex, history]);

  const loadArticles = async () => {
    try {
      const data = await invoke<HelpArticleSummary[]>('get_help_articles');
      setArticles(data);
    } catch (error) {
      console.error('Failed to load help articles:', error);
    }
  };

  const loadArticle = async (id: string, addToHistory = true) => {
    setLoading(true);
    try {
      const data = await invoke<HelpArticle | null>('get_help_article', { id });
      if (data) {
        setCurrentArticle(data);
        setFeedbackSubmitted(false);
        
        if (addToHistory) {
          const newHistory = history.slice(0, historyIndex + 1);
          newHistory.push(id);
          setHistory(newHistory);
          setHistoryIndex(newHistory.length - 1);
        }
      }
    } catch (error) {
      console.error('Failed to load article:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadContextHelp = async (context: string) => {
    try {
      const data = await invoke<HelpArticleSummary[]>('get_context_help', { context });
      if (data.length > 0) {
        loadArticle(data[0].id);
      }
    } catch (error) {
      console.error('Failed to load context help:', error);
    }
  };

  const handleSearch = useCallback(async (query: string) => {
    setSearchQuery(query);
    
    if (query.trim().length < 2) {
      setSearchResults([]);
      return;
    }

    try {
      const results = await invoke<SearchResult[]>('search_help', { query });
      setSearchResults(results);
    } catch (error) {
      console.error('Search failed:', error);
    }
  }, []);

  const goBack = () => {
    if (historyIndex > 0) {
      const newIndex = historyIndex - 1;
      setHistoryIndex(newIndex);
      loadArticle(history[newIndex], false);
    }
  };

  const goForward = () => {
    if (historyIndex < history.length - 1) {
      const newIndex = historyIndex + 1;
      setHistoryIndex(newIndex);
      loadArticle(history[newIndex], false);
    }
  };

  const submitFeedback = async (helpful: boolean) => {
    if (!currentArticle || feedbackSubmitted) return;

    try {
      await invoke('submit_help_feedback', {
        articleId: currentArticle.id,
        helpful,
        comment: null,
      });
      setFeedbackSubmitted(true);
    } catch (error) {
      console.error('Failed to submit feedback:', error);
    }
  };

  const filteredArticles = selectedCategory
    ? articles.filter((a) => a.category === selectedCategory)
    : articles;

  const renderHtmlContent = (html: string) => {
    return { __html: html };
  };

  if (!isOpen) return null;

  return createPortal(
    <div className="help-overlay" onClick={onClose}>
      <div className="help-modal" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div className="help-header">
          <div className="help-search">
            <span className="search-icon">🔍</span>
            <input
              ref={searchInputRef}
              type="text"
              placeholder="Search help..."
              value={searchQuery}
              onChange={(e) => handleSearch(e.target.value)}
            />
            {searchQuery && (
              <button className="clear-search" onClick={() => handleSearch('')}>
                ×
              </button>
            )}
          </div>
          <div className="help-nav">
            <button
              onClick={goBack}
              disabled={historyIndex <= 0}
              title="Back (Ctrl+[)"
            >
              ←
            </button>
            <button
              onClick={goForward}
              disabled={historyIndex >= history.length - 1}
              title="Forward (Ctrl+])"
            >
              →
            </button>
            <button onClick={onClose} title="Close (Esc)">
              ×
            </button>
          </div>
        </div>

        <div className="help-content">
          {/* Sidebar */}
          <aside className="help-sidebar">
            <div className="category-filters">
              <button
                className={!selectedCategory ? 'active' : ''}
                onClick={() => setSelectedCategory(null)}
              >
                All Articles
              </button>
              {(Object.keys(categoryConfig) as HelpCategory[]).map((cat) => (
                <button
                  key={cat}
                  className={selectedCategory === cat ? 'active' : ''}
                  onClick={() => setSelectedCategory(cat)}
                >
                  <span>{categoryConfig[cat].icon}</span>
                  {categoryConfig[cat].label}
                </button>
              ))}
            </div>

            <div className="article-list">
              {searchQuery && searchResults.length > 0 ? (
                <>
                  <h4>Search Results</h4>
                  {searchResults.map((result) => (
                    <button
                      key={result.id}
                      className={`article-item ${currentArticle?.id === result.id ? 'active' : ''}`}
                      onClick={() => loadArticle(result.id)}
                    >
                      <span className="article-icon">
                        {categoryConfig[result.category].icon}
                      </span>
                      <div className="article-info">
                        <span className="article-title">{result.title}</span>
                        <span className="article-summary">{result.summary}</span>
                      </div>
                    </button>
                  ))}
                </>
              ) : (
                <>
                  <h4>
                    {selectedCategory
                      ? categoryConfig[selectedCategory].label
                      : 'All Articles'}
                  </h4>
                  {filteredArticles.map((article) => (
                    <button
                      key={article.id}
                      className={`article-item ${currentArticle?.id === article.id ? 'active' : ''}`}
                      onClick={() => loadArticle(article.id)}
                    >
                      <span className="article-icon">
                        {categoryConfig[article.category].icon}
                      </span>
                      <div className="article-info">
                        <span className="article-title">{article.title}</span>
                        <span className="article-summary">{article.summary}</span>
                      </div>
                    </button>
                  ))}
                </>
              )}
            </div>
          </aside>

          {/* Article View */}
          <main className="help-article-view">
            {loading ? (
              <div className="loading">Loading...</div>
            ) : currentArticle ? (
              <>
                <div className="article-header">
                  <span
                    className="category-badge"
                    style={{
                      backgroundColor: categoryConfig[currentArticle.category].color,
                    }}
                  >
                    {categoryConfig[currentArticle.category].icon}{' '}
                    {categoryConfig[currentArticle.category].label}
                  </span>
                  <h1>{currentArticle.title}</h1>
                  {currentArticle.tags.length > 0 && (
                    <div className="article-tags">
                      {currentArticle.tags.map((tag) => (
                        <span key={tag} className="tag">
                          {tag}
                        </span>
                      ))}
                    </div>
                  )}
                </div>

                <div
                  className="article-body"
                  dangerouslySetInnerHTML={renderHtmlContent(
                    currentArticle.html_content || currentArticle.content
                  )}
                />

                <div className="article-feedback">
                  {feedbackSubmitted ? (
                    <span className="feedback-thanks">Thanks for your feedback!</span>
                  ) : (
                    <>
                      <span>Was this helpful?</span>
                      <button onClick={() => submitFeedback(true)}>👍 Yes</button>
                      <button onClick={() => submitFeedback(false)}>👎 No</button>
                    </>
                  )}
                </div>
              </>
            ) : (
              <div className="welcome-screen">
                <h2>Welcome to Super Punch-Out!! Editor Help</h2>
                <p>
                  Search for topics using the search bar above, or browse articles by
                  category in the sidebar.
                </p>
                <div className="quick-links">
                  <h4>Popular Topics</h4>
                  {articles.slice(0, 5).map((article) => (
                    <button
                      key={article.id}
                      onClick={() => loadArticle(article.id)}
                    >
                      {article.title}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </main>
        </div>
      </div>
    </div>,
    document.body
  );
};

export default HelpSystem;
