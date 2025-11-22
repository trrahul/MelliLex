import { afterEach, describe, expect, it, vi } from 'vitest';
import { ExportService, type WordExportPayload } from '../ExportService';
import { ERROR_MESSAGES } from '../../utils/errorHandler';

const mockApi = vi.hoisted(() => ({
  exportMarkdown: vi.fn(),
  exportToCapacities: vi.fn(),
}));

const mockMarkdownFormatter = vi.hoisted(() => ({
  format: vi.fn(),
}));

vi.mock('../api', () => ({
  api: mockApi,
}));

vi.mock('../MarkdownFormatter', () => ({
  MarkdownFormatter: mockMarkdownFormatter,
}));

const samplePayload: WordExportPayload = {
  header: {
    word: 'eloquent',
    pronunciation: 'ˈɛləkwənt',
    syllables: 'el-o-quent',
    origin: 'Latin eloquens',
    formality: { level: 'Formal', percentage: 78 },
    domains: [],
    tldr: 'Able to express yourself clearly and effectively.',
  },
};

const capacitiesConfig = {
  apiToken: 'cap-token',
  spaceId: 'space-123',
  defaultTags: ['Vocabulary'],
  noTimestamp: true,
};

afterEach(() => {
  vi.clearAllMocks();
});

describe('ExportService', () => {
  describe('exportToMarkdownFile', () => {
    it('delegates to API and returns resulting path', async () => {
      mockApi.exportMarkdown.mockResolvedValue('/exports/eloquent.md');

      const result = await ExportService.exportToMarkdownFile('eloquent', 'cloud', false);

      expect(mockApi.exportMarkdown).toHaveBeenCalledWith('eloquent', 'cloud', false);
      expect(result).toBe('/exports/eloquent.md');
    });
  });

  describe('exportToCapacities', () => {
    it('throws when configuration is missing token or spaceId', async () => {
      await expect(
        ExportService.exportToCapacities(samplePayload, { apiToken: '', spaceId: '', defaultTags: [], noTimestamp: false })
      ).rejects.toThrow(ERROR_MESSAGES.EXPORT_CONFIG_MISSING);
    });

    it('formats markdown and sends payload to Capacities API', async () => {
      mockMarkdownFormatter.format.mockReturnValue('# markdown doc');

      await ExportService.exportToCapacities(samplePayload, capacitiesConfig, {
        includeExploration: true,
        includeTimestamp: true,
      });

      expect(mockMarkdownFormatter.format).toHaveBeenCalledWith(samplePayload, {
        tags: capacitiesConfig.defaultTags,
        includeTimestamp: true,
        includeExploration: true,
      });

      expect(mockApi.exportToCapacities).toHaveBeenCalledWith(
        'cap-token',
        'space-123',
        '# markdown doc',
        true
      );
    });

    it('falls back to config-based timestamp preference when option omitted', async () => {
      mockMarkdownFormatter.format.mockReturnValue('doc');

      await ExportService.exportToCapacities(samplePayload, capacitiesConfig);

      expect(mockMarkdownFormatter.format).toHaveBeenCalledWith(samplePayload, {
        tags: capacitiesConfig.defaultTags,
        includeTimestamp: false,
        includeExploration: false,
      });
    });
  });
});
