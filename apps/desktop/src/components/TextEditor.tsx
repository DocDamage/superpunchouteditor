/**
 * Text/Dialog Editor - Main Component
 * 
 * Provides editing capabilities for in-game text:
 * - Cornerman advice texts
 * - Boxer intros (name, origin, record, rank, quote)
 * - Victory/defeat quotes
 * - Menu text
 * - Credits
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';
import { TextPreview } from './TextPreview';
import './TextEditor.css';

type TabType = 'cornerman' | 'intros' | 'victory' | 'menus' | 'credits';

// DTO Types matching the Rust backend
interface CornermanTextDto {
  id: number;
  boxer_key: string;
  round: number;
  condition: string;
  condition_value: number;
  text: string;
  byte_length: number;
  max_length: number;
  is_valid: boolean;
}

interface BoxerIntroResponse {
  boxer_key: string;
  name_text: string;
  origin_text: string;
  record_text: string;
  rank_text: string;
  intro_quote: string;
  validation: IntroValidation;
}

interface IntroValidation {
  name_valid: boolean;
  name_length: number;
  origin_valid: boolean;
  origin_length: number;
  record_valid: boolean;
  record_length: number;
  rank_valid: boolean;
  rank_length: number;
  quote_valid: boolean;
  quote_length: number;
  all_valid: boolean;
  unsupported_chars: string[];
}

interface VictoryQuoteDto {
  id: number;
  boxer_key: string;
  text: string;
  condition: string;
  condition_value: number;
  is_loss_quote: boolean;
  byte_length: number;
  max_length: number;
  is_valid: boolean;
}

interface MenuTextDto {
  id: string;
  category: string;
  text: string;
  byte_length: number;
  max_length: number;
  is_valid: boolean;
  is_modified: boolean;
  is_shared: boolean;
}

interface TextCondition {
  value: number;
  label: string;
}

interface TextEncodingInfo {
  supported_chars: string[];
  max_cornerman_length: number;
  max_victory_length: number;
  max_menu_length: number;
  max_intro_name_length: number;
  max_intro_origin_length: number;
  max_intro_record_length: number;
  max_intro_rank_length: number;
}

interface TextEditorProps {
  initialTab?: TabType;
  initialBoxerKey?: string;
}

export function TextEditor({ initialTab = 'cornerman', initialBoxerKey }: TextEditorProps) {
  const { boxers } = useStore();
  const [activeTab, setActiveTab] = useState<TabType>(initialTab);
  const [selectedBoxer, setSelectedBoxer] = useState<string>(initialBoxerKey || '');
  
  // Data states
  const [cornermanTexts, setCornermanTexts] = useState<CornermanTextDto[]>([]);
  const [boxerIntro, setBoxerIntro] = useState<BoxerIntroResponse | null>(null);
  const [victoryQuotes, setVictoryQuotes] = useState<VictoryQuoteDto[]>([]);
  const [menuTexts, setMenuTexts] = useState<MenuTextDto[]>([]);
  const [textConditions, setTextConditions] = useState<TextCondition[]>([]);
  const [encodingInfo, setEncodingInfo] = useState<TextEncodingInfo | null>(null);
  
  // UI states
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  
  // Editing states
  const [editingCornermanId, setEditingCornermanId] = useState<number | null>(null);
  const [editingCornermanText, setEditingCornermanText] = useState('');
  const [editingCornermanCondition, setEditingCornermanCondition] = useState(0);
  const [editingCornermanRound, setEditingCornermanRound] = useState(0);
  
  // Intro editing state
  const [editingIntro, setEditingIntro] = useState<Partial<BoxerIntroResponse>>({});
  
  // Preview state
  const [previewText, setPreviewText] = useState('');
  const [showPreview, setShowPreview] = useState(false);

  // Load text conditions
  const loadTextConditions = useCallback(async () => {
    try {
      const conditions = await invoke<TextCondition[]>('get_text_conditions');
      setTextConditions(conditions);
    } catch (err) {
      console.error('Failed to load text conditions:', err);
    }
  }, []);

  // Load encoding info
  const loadEncodingInfo = useCallback(async () => {
    try {
      const info = await invoke<TextEncodingInfo>('get_text_editor_encoding_info');
      setEncodingInfo(info);
    } catch (err) {
      console.error('Failed to load encoding info:', err);
    }
  }, []);

  // Load cornerman texts
  const loadCornermanTexts = useCallback(async () => {
    if (!selectedBoxer) return;
    
    try {
      setLoading(true);
      const texts = await invoke<CornermanTextDto[]>('get_cornerman_texts', {
        boxerKey: selectedBoxer,
      });
      setCornermanTexts(texts);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load cornerman texts');
    } finally {
      setLoading(false);
    }
  }, [selectedBoxer]);

  // Load boxer intro
  const loadBoxerIntro = useCallback(async () => {
    if (!selectedBoxer) return;
    
    try {
      setLoading(true);
      const intro = await invoke<BoxerIntroResponse>('get_boxer_intro', {
        boxerKey: selectedBoxer,
      });
      setBoxerIntro(intro);
      setEditingIntro(intro);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load boxer intro');
    } finally {
      setLoading(false);
    }
  }, [selectedBoxer]);

  // Load victory quotes
  const loadVictoryQuotes = useCallback(async () => {
    if (!selectedBoxer) return;
    
    try {
      setLoading(true);
      const quotes = await invoke<VictoryQuoteDto[]>('get_victory_quotes', {
        boxerKey: selectedBoxer,
      });
      setVictoryQuotes(quotes);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load victory quotes');
    } finally {
      setLoading(false);
    }
  }, [selectedBoxer]);

  // Load menu texts
  const loadMenuTexts = useCallback(async () => {
    try {
      setLoading(true);
      const texts = await invoke<MenuTextDto[]>('get_menu_texts', {
        category: null,
      });
      setMenuTexts(texts);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load menu texts');
    } finally {
      setLoading(false);
    }
  }, []);

  // Initial load
  useEffect(() => {
    loadTextConditions();
    loadEncodingInfo();
    loadMenuTexts();
  }, [loadTextConditions, loadEncodingInfo, loadMenuTexts]);

  // Load data when tab or boxer changes
  useEffect(() => {
    setError(null);
    
    switch (activeTab) {
      case 'cornerman':
        loadCornermanTexts();
        break;
      case 'intros':
        loadBoxerIntro();
        break;
      case 'victory':
        loadVictoryQuotes();
        break;
      case 'menus':
        loadMenuTexts();
        break;
      case 'credits':
        // TODO: Load credits
        break;
    }
  }, [activeTab, selectedBoxer, loadCornermanTexts, loadBoxerIntro, loadVictoryQuotes, loadMenuTexts]);

  // Update selected boxer when boxers load
  useEffect(() => {
    if (boxers.length > 0 && !selectedBoxer) {
      setSelectedBoxer(boxers[0].key);
    }
  }, [boxers, selectedBoxer]);

  // Start editing cornerman text
  const startEditingCornerman = (text: CornermanTextDto) => {
    setEditingCornermanId(text.id);
    setEditingCornermanText(text.text);
    setEditingCornermanCondition(text.condition_value);
    setEditingCornermanRound(text.round);
  };

  // Save cornerman text
  const saveCornermanText = async () => {
    if (editingCornermanId === null) return;
    
    try {
      setSaving(true);
      await invoke<CornermanTextDto>('update_cornerman_text', {
        request: {
          id: editingCornermanId,
          text: editingCornermanText,
          condition: editingCornermanCondition,
          round: editingCornermanRound,
        },
      });
      
      setEditingCornermanId(null);
      await loadCornermanTexts();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save cornerman text');
    } finally {
      setSaving(false);
    }
  };

  // Delete cornerman text
  const deleteCornermanText = async (id: number) => {
    if (!confirm('Delete this cornerman text?')) return;
    
    try {
      setSaving(true);
      await invoke('delete_cornerman_text', { id });
      await loadCornermanTexts();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete cornerman text');
    } finally {
      setSaving(false);
    }
  };

  // Add new cornerman text
  const addCornermanText = async () => {
    if (!selectedBoxer) return;
    
    try {
      setSaving(true);
      await invoke<CornermanTextDto>('add_cornerman_text', {
        boxerKey: selectedBoxer,
        text: 'New advice text...',
        condition: 7, // Random
        round: 0,
      });
      await loadCornermanTexts();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add cornerman text');
    } finally {
      setSaving(false);
    }
  };

  // Save boxer intro
  const saveBoxerIntro = async () => {
    if (!selectedBoxer || !editingIntro) return;
    
    try {
      setSaving(true);
      await invoke<BoxerIntroResponse>('update_boxer_intro', {
        request: {
          boxerKey: selectedBoxer,
          nameText: editingIntro.name_text,
          originText: editingIntro.origin_text,
          recordText: editingIntro.record_text,
          rankText: editingIntro.rank_text,
          introQuote: editingIntro.intro_quote,
        },
      });
      await loadBoxerIntro();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save boxer intro');
    } finally {
      setSaving(false);
    }
  };

  // Save victory quote
  const saveVictoryQuote = async (id: number, text: string) => {
    try {
      setSaving(true);
      await invoke<VictoryQuoteDto>('update_victory_quote', {
        request: { id, text },
      });
      await loadVictoryQuotes();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save victory quote');
    } finally {
      setSaving(false);
    }
  };

  // Save menu text
  const saveMenuText = async (id: string, text: string) => {
    try {
      setSaving(true);
      await invoke<MenuTextDto>('update_menu_text', {
        request: { id, text },
      });
      await loadMenuTexts();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save menu text');
    } finally {
      setSaving(false);
    }
  };

  // Preview text
  const handlePreviewText = async (text: string) => {
    setPreviewText(text);
    setShowPreview(true);
  };

  // Get byte count color
  const getByteCountColor = (current: number, max: number): string => {
    const ratio = current / max;
    if (ratio > 1) return 'var(--error)';
    if (ratio > 0.9) return 'var(--warning)';
    return 'var(--success)';
  };

  // Render boxer selector
  const renderBoxerSelector = () => (
    <div className="text-editor__boxer-select">
      <label>Boxer:</label>
      <select
        value={selectedBoxer}
        onChange={(e) => setSelectedBoxer(e.target.value)}
        disabled={loading}
      >
        <option value="">Select a boxer...</option>
        {boxers.map((boxer) => (
          <option key={boxer.key} value={boxer.key}>
            {boxer.name}
          </option>
        ))}
      </select>
    </div>
  );

  // Render cornerman tab
  const renderCornermanTab = () => (
    <div className="text-editor__tab-content">
      <div className="text-editor__header">
        {renderBoxerSelector()}
        <button
          className="text-editor__add-btn"
          onClick={addCornermanText}
          disabled={!selectedBoxer || saving}
        >
          + Add Text
        </button>
      </div>

      {selectedBoxer && (
        <div className="text-editor__list">
          {cornermanTexts.map((text) => (
            <div
              key={text.id}
              className={`text-editor__item ${!text.is_valid ? 'text-editor__item--invalid' : ''}`}
            >
              {editingCornermanId === text.id ? (
                <div className="text-editor__edit-form">
                  <div className="text-editor__form-row">
                    <label>Text:</label>
                    <textarea
                      value={editingCornermanText}
                      onChange={(e) => setEditingCornermanText(e.target.value)}
                      rows={2}
                      maxLength={encodingInfo?.max_cornerman_length || 40}
                    />
                  </div>
                  <div className="text-editor__form-row">
                    <label>Condition:</label>
                    <select
                      value={editingCornermanCondition}
                      onChange={(e) => setEditingCornermanCondition(Number(e.target.value))}
                    >
                      {textConditions.map((cond) => (
                        <option key={cond.value} value={cond.value}>
                          {cond.label}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div className="text-editor__form-row">
                    <label>Round:</label>
                    <select
                      value={editingCornermanRound}
                      onChange={(e) => setEditingCornermanRound(Number(e.target.value))}
                    >
                      <option value={0}>Any Round</option>
                      <option value={1}>Round 1</option>
                      <option value={2}>Round 2</option>
                      <option value={3}>Round 3</option>
                    </select>
                  </div>
                  <div className="text-editor__byte-count">
                    <span
                      style={{
                        color: getByteCountColor(
                          editingCornermanText.length,
                          encodingInfo?.max_cornerman_length || 40
                        ),
                      }}
                    >
                      Chars: {editingCornermanText.length}/{encodingInfo?.max_cornerman_length || 40}
                    </span>
                  </div>
                  <div className="text-editor__actions">
                    <button
                      className="text-editor__save-btn"
                      onClick={saveCornermanText}
                      disabled={saving}
                    >
                      Save
                    </button>
                    <button
                      className="text-editor__cancel-btn"
                      onClick={() => setEditingCornermanId(null)}
                      disabled={saving}
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              ) : (
                <>
                  <div className="text-editor__item-content">
                    <div className="text-editor__item-text">&ldquo;{text.text}&rdquo;</div>
                    <div className="text-editor__item-meta">
                      <span className="text-editor__condition">{text.condition}</span>
                      {text.round > 0 && (
                        <span className="text-editor__round">Round {text.round}</span>
                      )}
                      <span
                        className="text-editor__byte-count"
                        style={{
                          color: getByteCountColor(text.byte_length, text.max_length),
                        }}
                      >
                        {text.byte_length}/{text.max_length} bytes
                      </span>
                    </div>
                  </div>
                  <div className="text-editor__item-actions">
                    <button
                      className="text-editor__preview-btn"
                      onClick={() => handlePreviewText(text.text)}
                      title="Preview"
                    >
                      👁
                    </button>
                    <button
                      className="text-editor__edit-btn"
                      onClick={() => startEditingCornerman(text)}
                      disabled={saving}
                    >
                      Edit
                    </button>
                    <button
                      className="text-editor__delete-btn"
                      onClick={() => deleteCornermanText(text.id)}
                      disabled={saving}
                    >
                      Delete
                    </button>
                  </div>
                </>
              )}
            </div>
          ))}

          {cornermanTexts.length === 0 && !loading && (
            <div className="text-editor__empty">
              No cornerman texts for this boxer.
            </div>
          )}
        </div>
      )}
    </div>
  );

  // Render intros tab
  const renderIntrosTab = () => (
    <div className="text-editor__tab-content">
      <div className="text-editor__header">
        {renderBoxerSelector()}
      </div>

      {boxerIntro && editingIntro && (
        <div className="text-editor__intro-form">
          <div className="text-editor__intro-field">
            <label>Name:</label>
            <input
              type="text"
              value={editingIntro.name_text || ''}
              onChange={(e) => setEditingIntro({ ...editingIntro, name_text: e.target.value })}
              maxLength={encodingInfo?.max_intro_name_length || 16}
            />
            <span
              className="text-editor__field-count"
              style={{
                color: getByteCountColor(
                  boxerIntro.validation.name_length,
                  encodingInfo?.max_intro_name_length || 16
                ),
              }}
            >
              {boxerIntro.validation.name_length}/{encodingInfo?.max_intro_name_length || 16}
            </span>
            {!boxerIntro.validation.name_valid && (
              <span className="text-editor__field-error">Invalid</span>
            )}
          </div>

          <div className="text-editor__intro-field">
            <label>Origin:</label>
            <input
              type="text"
              value={editingIntro.origin_text || ''}
              onChange={(e) => setEditingIntro({ ...editingIntro, origin_text: e.target.value })}
              maxLength={encodingInfo?.max_intro_origin_length || 32}
            />
            <span
              className="text-editor__field-count"
              style={{
                color: getByteCountColor(
                  boxerIntro.validation.origin_length,
                  encodingInfo?.max_intro_origin_length || 32
                ),
              }}
            >
              {boxerIntro.validation.origin_length}/{encodingInfo?.max_intro_origin_length || 32}
            </span>
          </div>

          <div className="text-editor__intro-field">
            <label>Record:</label>
            <input
              type="text"
              value={editingIntro.record_text || ''}
              onChange={(e) => setEditingIntro({ ...editingIntro, record_text: e.target.value })}
              maxLength={encodingInfo?.max_intro_record_length || 20}
            />
            <span
              className="text-editor__field-count"
              style={{
                color: getByteCountColor(
                  boxerIntro.validation.record_length,
                  encodingInfo?.max_intro_record_length || 20
                ),
              }}
            >
              {boxerIntro.validation.record_length}/{encodingInfo?.max_intro_record_length || 20}
            </span>
          </div>

          <div className="text-editor__intro-field">
            <label>Rank:</label>
            <input
              type="text"
              value={editingIntro.rank_text || ''}
              onChange={(e) => setEditingIntro({ ...editingIntro, rank_text: e.target.value })}
              maxLength={encodingInfo?.max_intro_rank_length || 24}
            />
            <span
              className="text-editor__field-count"
              style={{
                color: getByteCountColor(
                  boxerIntro.validation.rank_length,
                  encodingInfo?.max_intro_rank_length || 24
                ),
              }}
            >
              {boxerIntro.validation.rank_length}/{encodingInfo?.max_intro_rank_length || 24}
            </span>
          </div>

          <div className="text-editor__intro-field">
            <label>Quote:</label>
            <textarea
              value={editingIntro.intro_quote || ''}
              onChange={(e) => setEditingIntro({ ...editingIntro, intro_quote: e.target.value })}
              rows={3}
              maxLength={encodingInfo?.max_victory_length || 50}
            />
            <span
              className="text-editor__field-count"
              style={{
                color: getByteCountColor(
                  boxerIntro.validation.quote_length,
                  encodingInfo?.max_victory_length || 50
                ),
              }}
            >
              {boxerIntro.validation.quote_length}/{encodingInfo?.max_victory_length || 50}
            </span>
          </div>

          {boxerIntro.validation.unsupported_chars.length > 0 && (
            <div className="text-editor__unsupported-warning">
              Unsupported characters: {boxerIntro.validation.unsupported_chars.join(', ')}
            </div>
          )}

          <div className="text-editor__actions">
            <button
              className="text-editor__save-btn"
              onClick={saveBoxerIntro}
              disabled={saving || !boxerIntro.validation.all_valid}
            >
              Save Changes
            </button>
            <button
              className="text-editor__preview-btn"
              onClick={() => handlePreviewText(editingIntro.intro_quote || '')}
            >
              Preview Quote
            </button>
          </div>
        </div>
      )}
    </div>
  );

  // Render victory quotes tab
  const renderVictoryTab = () => (
    <div className="text-editor__tab-content">
      <div className="text-editor__header">
        {renderBoxerSelector()}
      </div>

      {selectedBoxer && (
        <div className="text-editor__list">
          {victoryQuotes.map((quote) => (
            <div
              key={quote.id}
              className={`text-editor__item ${!quote.is_valid ? 'text-editor__item--invalid' : ''}`}
            >
              <div className="text-editor__item-content">
                <div className="text-editor__item-text">&ldquo;{quote.text}&rdquo;</div>
                <div className="text-editor__item-meta">
                  <span className="text-editor__condition">{quote.condition}</span>
                  {quote.is_loss_quote && (
                    <span className="text-editor__loss-tag">Loss Quote</span>
                  )}
                  <span
                    className="text-editor__byte-count"
                    style={{
                      color: getByteCountColor(quote.byte_length, quote.max_length),
                    }}
                  >
                    {quote.byte_length}/{quote.max_length} bytes
                  </span>
                </div>
              </div>
              <div className="text-editor__item-actions">
                <button
                  className="text-editor__edit-btn"
                  onClick={() => {
                    const newText = prompt('Edit quote:', quote.text);
                    if (newText !== null && newText !== quote.text) {
                      saveVictoryQuote(quote.id, newText);
                    }
                  }}
                  disabled={saving}
                >
                  Edit
                </button>
              </div>
            </div>
          ))}

          {victoryQuotes.length === 0 && !loading && (
            <div className="text-editor__empty">
              No victory quotes for this boxer.
            </div>
          )}
        </div>
      )}
    </div>
  );

  // Render menu texts tab
  const renderMenusTab = () => (
    <div className="text-editor__tab-content">
      <div className="text-editor__list">
        {menuTexts.map((menu) => (
          <div
            key={menu.id}
            className={`text-editor__item ${!menu.is_valid ? 'text-editor__item--invalid' : ''} ${menu.is_modified ? 'text-editor__item--modified' : ''}`}
          >
            <div className="text-editor__item-content">
              <div className="text-editor__item-id">{menu.id}</div>
              <div className="text-editor__item-text">{menu.text}</div>
              <div className="text-editor__item-meta">
                <span className="text-editor__category">{menu.category}</span>
                {menu.is_shared && (
                  <span className="text-editor__shared-tag">Shared</span>
                )}
                <span
                  className="text-editor__byte-count"
                  style={{
                    color: getByteCountColor(menu.byte_length, menu.max_length),
                  }}
                >
                  {menu.byte_length}/{menu.max_length} bytes
                </span>
              </div>
            </div>
            <div className="text-editor__item-actions">
              <button
                className="text-editor__edit-btn"
                onClick={() => {
                  const newText = prompt('Edit menu text:', menu.text);
                  if (newText !== null && newText !== menu.text) {
                    saveMenuText(menu.id, newText);
                  }
                }}
                disabled={saving}
              >
                Edit
              </button>
            </div>
          </div>
        ))}

        {menuTexts.length === 0 && !loading && (
          <div className="text-editor__empty">
            No menu texts available.
          </div>
        )}
      </div>
    </div>
  );

  // Render credits tab (placeholder)
  const renderCreditsTab = () => (
    <div className="text-editor__tab-content">
      <div className="text-editor__placeholder">
        <p>Credits editor coming soon.</p>
        <p>This will allow editing of the end-game credits text.</p>
      </div>
    </div>
  );

  return (
    <div className="text-editor">
      <div className="text-editor__header-bar">
        <h2>Text & Dialog Editor</h2>
        {showPreview && (
          <button
            className="text-editor__preview-toggle"
            onClick={() => setShowPreview(false)}
          >
            Hide Preview
          </button>
        )}
      </div>

      <div className="text-editor__tabs">
        <button
          className={`text-editor__tab ${activeTab === 'cornerman' ? 'text-editor__tab--active' : ''}`}
          onClick={() => setActiveTab('cornerman')}
        >
          Cornerman
        </button>
        <button
          className={`text-editor__tab ${activeTab === 'intros' ? 'text-editor__tab--active' : ''}`}
          onClick={() => setActiveTab('intros')}
        >
          Intros
        </button>
        <button
          className={`text-editor__tab ${activeTab === 'victory' ? 'text-editor__tab--active' : ''}`}
          onClick={() => setActiveTab('victory')}
        >
          Victory
        </button>
        <button
          className={`text-editor__tab ${activeTab === 'menus' ? 'text-editor__tab--active' : ''}`}
          onClick={() => setActiveTab('menus')}
        >
          Menus
        </button>
        <button
          className={`text-editor__tab ${activeTab === 'credits' ? 'text-editor__tab--active' : ''}`}
          onClick={() => setActiveTab('credits')}
        >
          Credits
        </button>
      </div>

      {error && (
        <div className="text-editor__error">
          {error}
        </div>
      )}

      <div className="text-editor__content">
        {activeTab === 'cornerman' && renderCornermanTab()}
        {activeTab === 'intros' && renderIntrosTab()}
        {activeTab === 'victory' && renderVictoryTab()}
        {activeTab === 'menus' && renderMenusTab()}
        {activeTab === 'credits' && renderCreditsTab()}
      </div>

      {showPreview && (
        <TextPreview
          text={previewText}
          onClose={() => setShowPreview(false)}
        />
      )}

      {loading && (
        <div className="text-editor__loading">
          Loading...
        </div>
      )}
    </div>
  );
}
