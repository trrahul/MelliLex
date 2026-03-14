import type {
  CapacitiesConfig,
} from '../types';
import { ERROR_MESSAGES } from '../utils/errorHandler';
import { api } from './api';
import { 
  MarkdownFormatter, 
  type WordExportPayload,
  type MarkdownFormatOptions,
} from './MarkdownFormatter';

export type { WordExportPayload };

export class ExportService {
  static async exportToMarkdownFile(
    word: string,
    provider: string,
    includeTimestamp: boolean = true
  ): Promise<string> {
    // Backend will fetch from cache and generate markdown
    return await api.exportMarkdown(word, provider, includeTimestamp);
  }

  static async exportPhraseToMarkdownFile(
    phrase: string,
    provider: string,
    includeTimestamp: boolean = true
  ): Promise<string> {
    // Backend will fetch from cache and generate markdown (same as words)
    return await api.exportPhraseMarkdown(phrase, provider, includeTimestamp);
  }

  static async exportToCapacities(
    payload: WordExportPayload,
    config?: CapacitiesConfig,
    options: MarkdownFormatOptions = {}
  ): Promise<void> {
    console.log('[ExportService] Starting Capacities export');
    
    if (!config?.apiToken || !config?.spaceId) {
      console.error('[ExportService] Missing configuration:', { 
        hasApiToken: !!config?.apiToken, 
        hasSpaceId: !!config?.spaceId 
      });
      throw new Error(ERROR_MESSAGES.EXPORT_CONFIG_MISSING);
    }

    console.log('[ExportService] Building markdown document');
    // Delegate formatting to MarkdownFormatter (Capacities needs custom format with exploration)
    const markdown = MarkdownFormatter.format(payload, {
      tags: config.defaultTags,
      includeTimestamp: options.includeTimestamp ?? !config.noTimestamp,
      includeExploration: options.includeExploration ?? false,
    });
    
    console.log('[ExportService] Markdown generated:', { length: markdown.length });

    try {
      await api.exportToCapacities(
        config.apiToken,
        config.spaceId,
        markdown,
        config.noTimestamp ?? false
      );
      console.log('[ExportService] Export completed successfully');
    } catch (error) {
      console.error('[ExportService] Export failed:', error);
      throw error;
    }
  }

  static async exportPhraseToCapacities(
    markdown: string,
    config: CapacitiesConfig
  ): Promise<void> {
    console.log('[ExportService] Starting Capacities export for phrase');

    if (!config?.apiToken || !config?.spaceId) {
      console.error('[ExportService] Missing configuration:', {
        hasApiToken: !!config?.apiToken,
        hasSpaceId: !!config?.spaceId
      });
      throw new Error(ERROR_MESSAGES.EXPORT_CONFIG_MISSING);
    }

    try {
      await api.exportToCapacities(
        config.apiToken,
        config.spaceId,
        markdown,
        config.noTimestamp ?? false
      );
      console.log('[ExportService] Phrase export completed successfully');
    } catch (error) {
      console.error('[ExportService] Phrase export failed:', error);
      throw error;
    }
  }
}
