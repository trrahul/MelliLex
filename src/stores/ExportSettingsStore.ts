import { makeAutoObservable, runInAction } from 'mobx';
import type { ExportSettings, CapacitiesConfig } from '../types';
import { getErrorMessage, ERROR_MESSAGES } from '../utils/errorHandler';

export interface ExportSettingsStoreCallbacks {
  onUpdate: (exportSettings: ExportSettings) => Promise<void>;
}

/**
 * Manages export configuration state and operations.
 * Single Responsibility: Export settings only.
 */
export class ExportSettingsStore {
  // UI state
  enabled = false;
  spaceId = '';
  apiToken = '';
  tags = '';
  includeExploration = false;
  includeTimestamp = true;
  
  // Operation state
  saving = false;
  saved = false;
  error: string | null = null;
  
  // Original values for change detection
  private originalValues: {
    enabled: boolean;
    spaceId: string;
    apiToken: string;
    tags: string;
    includeExploration: boolean;
    includeTimestamp: boolean;
  } | null = null;

  private callbacks: ExportSettingsStoreCallbacks;

  constructor(callbacks: ExportSettingsStoreCallbacks) {
    this.callbacks = callbacks;
    makeAutoObservable(this);
  }

  loadFromSettings(exportSettings?: ExportSettings) {
    const capacities = exportSettings?.capacities;
    const hasCapacities = Boolean(capacities?.apiToken && capacities?.spaceId);
    
    runInAction(() => {
      this.enabled = hasCapacities;
      this.spaceId = capacities?.spaceId ?? '';
      this.apiToken = capacities?.apiToken ?? '';
      this.tags = capacities?.defaultTags?.join(', ') ?? '';
      this.includeExploration = exportSettings?.includeExploration ?? false;
      this.includeTimestamp = !capacities?.noTimestamp;
      
      // Store original values
      this.originalValues = {
        enabled: this.enabled,
        spaceId: this.spaceId,
        apiToken: this.apiToken,
        tags: this.tags,
        includeExploration: this.includeExploration,
        includeTimestamp: this.includeTimestamp,
      };
      
      this.saved = false;
      this.error = null;
    });
  }

  get hasChanges(): boolean {
    if (!this.originalValues) return false;
    
    return (
      this.enabled !== this.originalValues.enabled ||
      this.spaceId !== this.originalValues.spaceId ||
      this.apiToken !== this.originalValues.apiToken ||
      this.tags !== this.originalValues.tags ||
      this.includeExploration !== this.originalValues.includeExploration ||
      this.includeTimestamp !== this.originalValues.includeTimestamp
    );
  }

  get isValid(): boolean {
    if (!this.enabled) return true;
    
    const hasSpaceId = this.spaceId.trim().length > 0;
    const hasApiToken = this.apiToken.trim().length > 0;
    
    return hasSpaceId && hasApiToken;
  }

  get validationError(): string | null {
    if (!this.enabled) return null;
    
    if (!this.spaceId.trim()) {
      return 'Space ID is required';
    }
    
    if (!this.apiToken.trim()) {
      return 'API Token is required';
    }
    
    return null;
  }

  get canSave(): boolean {
    return !this.saving && this.hasChanges && this.isValid;
  }

  private buildExportSettings(): ExportSettings {
    const defaultTags = this.tags
      .split(',')
      .map((tag) => tag.trim())
      .filter(Boolean);

    return {
      includeExploration: this.includeExploration,
      capacities: this.enabled ? {
        apiToken: this.apiToken.trim(),
        spaceId: this.spaceId.trim(),
        defaultTags,
        noTimestamp: !this.includeTimestamp,
      } : undefined,
    };
  }

  async save() {
    if (!this.canSave) {
      console.warn('[ExportSettingsStore] Cannot save - validation failed or no changes');
      return;
    }

    this.saving = true;
    this.error = null;
    this.saved = false;

    try {
      const exportSettings = this.buildExportSettings();
      
      // Delegate to parent store's update method
      await this.callbacks.onUpdate(exportSettings);
      
      runInAction(() => {
        // Update original values after successful save
        this.originalValues = {
          enabled: this.enabled,
          spaceId: this.spaceId,
          apiToken: this.apiToken,
          tags: this.tags,
          includeExploration: this.includeExploration,
          includeTimestamp: this.includeTimestamp,
        };
        
        this.saving = false;
        this.saved = true;
        
        // Reset saved state after 2 seconds
        setTimeout(() => {
          runInAction(() => {
            this.saved = false;
          });
        }, 2000);
      });
    } catch (err) {
      runInAction(() => {
        this.error = getErrorMessage(err, ERROR_MESSAGES.UPDATE_SETTINGS_FAILED);
        this.saving = false;
        this.saved = false;
      });
    }
  }

  // Setters for UI bindings
  setEnabled(value: boolean) {
    this.enabled = value;
  }

  setSpaceId(value: string) {
    this.spaceId = value;
  }

  setApiToken(value: string) {
    this.apiToken = value;
  }

  setTags(value: string) {
    this.tags = value;
  }

  setIncludeExploration(value: boolean) {
    this.includeExploration = value;
  }

  setIncludeTimestamp(value: boolean) {
    this.includeTimestamp = value;
  }

  /**
   * Get current Capacities config for export operations
   */
  get capacitiesConfig(): CapacitiesConfig | undefined {
    if (!this.enabled || !this.isValid) return undefined;

    const defaultTags = this.tags
      .split(',')
      .map((tag) => tag.trim())
      .filter(Boolean);

    return {
      apiToken: this.apiToken.trim(),
      spaceId: this.spaceId.trim(),
      defaultTags,
      noTimestamp: !this.includeTimestamp,
    };
  }
}
