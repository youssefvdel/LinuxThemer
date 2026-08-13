import {
  ArrowLeft,
  Check,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Download,
  Heart,
  Search,
  Trash2,
} from "lucide-react";

interface IconProps {
  size?: number;
}

export function SearchIcon({ size = 15 }: IconProps) {
  return <Search size={size} />;
}

export function DownloadIcon({ size = 14 }: IconProps) {
  return <Download size={size} />;
}

export function CheckIcon({ size = 13 }: IconProps) {
  return <Check size={size} />;
}

export function ChevronIcon({ size = 13 }: IconProps) {
  return <ChevronDown size={size} />;
}

export function LeftIcon({ size = 14 }: IconProps) {
  return <ChevronLeft size={size} />;
}

export function RightIcon({ size = 14 }: IconProps) {
  return <ChevronRight size={size} />;
}

export function BackIcon({ size = 16 }: IconProps) {
  return <ArrowLeft size={size} />;
}

export function HeartIcon({ size = 14, filled = false }: IconProps & { filled?: boolean }) {
  return <Heart size={size} fill={filled ? "currentColor" : "none"} />;
}

export function TrashIcon({ size = 13 }: IconProps) {
  return <Trash2 size={size} />;
}
