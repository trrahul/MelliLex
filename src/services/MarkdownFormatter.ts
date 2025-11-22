import type {
  DomainExploration,
  FormalityAlternative,
  MistakeItem,
  PracticeExercise,
  WordSection1Header,
  WordSection2Meanings,
  WordSection3Related,
  UsagePattern,
} from '../types';
import { ERROR_MESSAGES } from '../utils/errorHandler';

export interface WordExportPayload {
  header?: WordSection1Header | null;
  meanings?: WordSection2Meanings | null;
  related?: WordSection3Related | null;
  exploration?: ExplorationExportPayload | null;
}

export interface ExplorationExportPayload {
  formality?: {
    percentage?: number | null;
    alternatives?: FormalityAlternative[];
  };
  domains?: DomainExploration[];
  usage?: UsagePattern[];
  practice?: PracticeExercise[];
  mistakes?: MistakeItem[];
  customContext?: {
    label?: string;
    examples: string[];
  };
}

export interface MarkdownFormatOptions {
  includeTimestamp?: boolean;
  tags?: string[];
  includeExploration?: boolean;
}

const ensureHeader = (payload: WordExportPayload): WordSection1Header => {
  if (!payload.header) {
    throw new Error(ERROR_MESSAGES.EXPORT_NOT_READY);
  }
  return payload.header;
};

/**
 * Formats word data into markdown with YAML frontmatter.
 * Single Responsibility: Markdown document generation only.
 */
export class MarkdownFormatter {
  static format(
    payload: WordExportPayload,
    options: MarkdownFormatOptions = {}
  ): string {
    const header = ensureHeader(payload);
    const lines: string[] = [];

    // Frontmatter metadata
    lines.push('---');
    lines.push(`word: ${header.word}`);
    lines.push(`date: ${new Date().toISOString().split('T')[0]}`);
    lines.push(`formality: ${header.formality.percentage}%`);
    lines.push(`origin: ${header.origin}`);
    if (options.tags?.length) {
      const tags = options.tags.map(t => t.trim().replace(/^#/, '')).filter(Boolean);
      lines.push(`tags: [${tags.join(', ')}]`);
    }
    lines.push('---');
    lines.push('');

    // Title and metadata
    lines.push(`# ${header.word}`);
    lines.push('');
    lines.push(`**Pronunciation:** /${header.pronunciation}/`);
    lines.push(`**Part of Speech:** ${payload.meanings?.meanings?.[0]?.partOfSpeech || 'N/A'}`);
    lines.push(`**Formality:** ${header.formality.percentage}% ${header.formality.level}`);
    lines.push(`**Origin:** ${header.origin}`);
    lines.push('');
    lines.push('---');
    lines.push('');

    // TL;DR
    lines.push('## TL;DR');
    lines.push('');
    lines.push(`> **${header.tldr}**`);
    lines.push('');
    lines.push('---');
    lines.push('');

    // Meanings
    if (payload.meanings?.meanings?.length) {
      lines.push('## Meanings');
      lines.push('');
      payload.meanings.meanings.forEach((meaning) => {
        lines.push(`### ${meaning.number}. ${meaning.definition}`);
        lines.push('');
        if (meaning.examples?.length) {
          meaning.examples.forEach((example) => {
            lines.push(`${example}`);
          });
          lines.push('');
        }
        if (meaning.memoryTip) {
          lines.push(`**Memory Tip:** ${meaning.memoryTip}`);
          lines.push('');
        }
      });
      lines.push('---');
      lines.push('');
    }

    // Vocabulary Network
    const hasRelated = payload.related?.synonyms?.length || 
                       payload.related?.antonyms?.length || 
                       payload.related?.collocations?.length;
    
    if (hasRelated) {
      lines.push('## Vocabulary Network');
      lines.push('');
      
      if (payload.related?.synonyms?.length) {
        lines.push(`**Synonyms**`);
        lines.push(payload.related.synonyms.join(' • '));
        lines.push('');
      }
      
      if (payload.related?.antonyms?.length) {
        lines.push(`**Antonyms**`);
        lines.push(payload.related.antonyms.join(' • '));
        lines.push('');
      }
      
      if (payload.related?.collocations?.length) {
        lines.push(`**Collocations**`);
        payload.related.collocations.forEach((collocation) => {
          lines.push(`- ${collocation.phrase}`);
        });
        lines.push('');
      }
      
      lines.push('---');
      lines.push('');
    }

    // Formality Guide
    if (options.includeExploration && payload.exploration?.formality) {
      const { alternatives } = payload.exploration.formality;
      if (alternatives && alternatives.length > 0) {
        lines.push('## Formality Guide');
        lines.push('');
        lines.push('| Formality | Alternative | Context |');
        lines.push('|-----------|-------------|---------|');
        
        alternatives.forEach((alt) => {
          const register = alt.level || '';
          const alternative = alt.word || '';
          const context = alt.context || '';
          lines.push(`| **${register}** | ${alternative} | ${context} |`);
        });
        
        lines.push('');
        lines.push('---');
        lines.push('');
      }
    }

    // Domain-Specific Usage
    if (options.includeExploration && payload.exploration?.domains?.length) {
      lines.push('## Domain-Specific Usage');
      lines.push('');
      
      payload.exploration.domains.forEach((domain) => {
        lines.push(`### ${domain.domain}`);
        if (domain.examples?.length) {
          domain.examples.forEach((example) => {
            lines.push(`"${example}"`);
          });
        }
        lines.push('');
      });
      
      lines.push('---');
      lines.push('');
    }

    // Common Pitfalls
    if (options.includeExploration && payload.exploration?.mistakes?.length) {
      lines.push('## Common Pitfalls');
      lines.push('');
      
      payload.exploration.mistakes.forEach((mistake) => {
        lines.push(`**Incorrect (${mistake.type}):** ${mistake.incorrectUsage}`);
        lines.push(`**Correct:** ${mistake.correction}`);
        lines.push('');
      });
      
      lines.push('---');
      lines.push('');
    }

    // Usage Patterns
    if (options.includeExploration && payload.exploration?.usage?.length) {
      lines.push('## Grammar Patterns');
      lines.push('');
      
      payload.exploration.usage.forEach((pattern) => {
        lines.push(`**${pattern.template}**`);
        lines.push(`${pattern.description}`);
        if (pattern.examples?.length) {
          pattern.examples.forEach((example) => {
            lines.push(`- ${example}`);
          });
        }
        lines.push('');
      });
      
      lines.push('---');
      lines.push('');
    }

    // Practice
    if (options.includeExploration && payload.exploration?.practice?.length) {
      lines.push('## Practice');
      lines.push('');
      
      payload.exploration.practice.forEach((exercise, index) => {
        lines.push(`**${index + 1}. ${exercise.question}**`);
        lines.push('');
        exercise.options.forEach((option) => {
          const marker = option === exercise.correctAnswer ? 'x' : ' ';
          lines.push(`   [${marker}] ${option}`);
        });
        if (exercise.explanation) {
          lines.push('');
          lines.push(`   *${exercise.explanation}*`);
        }
        lines.push('');
      });
      
      lines.push('---');
      lines.push('');
    }

    // Footer
    lines.push('---');
    lines.push('');
    const timestamp = options.includeTimestamp !== false ? 
      new Date().toISOString().split('T')[0] : '';
    lines.push(`*Generated by MelliLex${timestamp ? ' · ' + timestamp : ''}*`);

    return lines.join('\n').trim() + '\n';
  }
}
