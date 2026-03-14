import type React from 'react';
import type { PhraseType, PhraseRegion } from '../../types';
import { getPhraseTypeDisplay } from '../../types';
import { cn } from '../../lib/utils';
import {
  Flag,
  Globe,
  Handshake,
  Lightbulb,
  Link2,
  MessageSquare,
  Mic2,
  ScrollText,
} from 'lucide-react';

interface Props {
  type: PhraseType;
  className?: string;
}

const typeStyles: Record<PhraseType, string> = {
  idiom: 'bg-purple-50 text-purple-900 border-purple-500',
  proverb: 'bg-amber-50 text-amber-900 border-amber-500',
  phrasalVerb: 'bg-blue-50 text-blue-900 border-blue-500',
  collocation: 'bg-green-50 text-green-900 border-green-500',
  expression: 'bg-pink-50 text-pink-900 border-pink-500',
  saying: 'bg-orange-50 text-orange-900 border-orange-500',
};

const typeIcons: Record<PhraseType, React.ComponentType<{ className?: string }>> = {
  idiom: Lightbulb,
  proverb: ScrollText,
  phrasalVerb: Link2,
  collocation: Handshake,
  expression: MessageSquare,
  saying: Mic2,
};

export const PhraseTypeBadge = ({ type, className }: Props) => {
  const Icon = typeIcons[type];
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium uppercase tracking-wide border',
        typeStyles[type],
        className
      )}
    >
      <Icon className="h-3.5 w-3.5" />
      <span>{getPhraseTypeDisplay(type)}</span>
    </span>
  );
};

interface RegionBadgeProps {
  region: PhraseRegion;
  className?: string;
}

const regionMeta: Record<
  PhraseRegion,
  { label: string; icon: React.ComponentType<{ className?: string }> }
> = {
  universal: { label: 'Universal', icon: Globe },
  american: { label: 'American', icon: Flag },
  british: { label: 'British', icon: Flag },
  australian: { label: 'Australian', icon: Flag },
};

export const RegionBadge = ({ region, className }: RegionBadgeProps) => {
  const { label, icon: Icon } = regionMeta[region];
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-muted text-muted-foreground border border-border',
        className
      )}
    >
      <Icon className="h-3.5 w-3.5" />
      <span>{label}</span>
    </span>
  );
};

interface FormalityBadgeProps {
  level: string;
  className?: string;
}

export const FormalityBadge = ({ level, className }: FormalityBadgeProps) => {
  const isInformal = level.toLowerCase().includes('informal');
  const isFormal = level.toLowerCase().includes('formal') && !isInformal;
  
  return (
    <span
      className={cn(
        'inline-flex items-center px-3 py-1 rounded-full text-xs font-medium border',
        isFormal && 'bg-blue-50 text-blue-900 border-blue-500',
        isInformal && 'bg-orange-50 text-orange-900 border-orange-500',
        !isFormal && !isInformal && 'bg-green-50 text-green-900 border-green-500',
        className
      )}
    >
      {level}
    </span>
  );
};
