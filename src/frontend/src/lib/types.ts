// Core types matching Rust backend models

export interface Bookmark {
  id: string;
  tweet_url: string;
  content: string;
  note_text: string | null;
  tweeted_at: string;
  imported_at: string;
  author_handle: string;
  author_name: string;
  author_profile_url: string | null;
  author_profile_image: string | null;
  tags: string[];
  comments: string | null;
  media: Media[];
  is_favorite: boolean;
}

export interface Media {
  url: string;
  media_type: 'Image' | 'Video' | 'Gif' | 'Unknown';
}

export interface BookmarkStats {
  total_bookmarks: number;
  unique_authors: number;
  unique_tags: number;
  favorite_bookmarks: number;
  earliest_date: string | null;
  latest_date: string | null;
  top_tags: [string, number][];
}

export interface SearchFilters {
  query?: string;
  tags?: string[];
  author?: string;
  date_from?: string;
  date_to?: string;
  has_media?: boolean;
  favorites_only?: boolean;
}

export type ViewMode = 'all' | 'favorites' | 'recent';

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  offset: number;
  limit: number;
  has_more: boolean;
}

export interface LinkPreview {
  url: string;
  final_url: string;
  title: string | null;
  description: string | null;
  image_url: string | null;
  site_name: string | null;
}

