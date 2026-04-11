import { describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const projectRoot = resolve(import.meta.dir, '..');

function fileExists(relativePath: string): boolean {
  return existsSync(resolve(projectRoot, relativePath));
}

function read(relativePath: string): string {
  return readFileSync(resolve(projectRoot, relativePath), 'utf8');
}

describe('frontend remake verification guards', () => {
  test('deletes the old frontend shell and stats references', () => {
    expect(fileExists('src/routes/stats/+page.svelte')).toBe(false);
    expect(fileExists('src/lib/components/Header.svelte')).toBe(false);
    expect(fileExists('src/lib/components/Sidebar.svelte')).toBe(false);
    expect(fileExists('src/lib/components/SearchBar.svelte')).toBe(false);
    expect(fileExists('src/lib/components/ImportModal.svelte')).toBe(false);

    const readme = read('../../README.md');
    expect(readme).not.toContain('Stats dashboard');
  });

  test('keeps the library route orchestration-only and wired to the new shell', () => {
    const page = read('src/routes/+page.svelte');

    expect(page).toContain("import LibraryWorkspace from '$lib/components/LibraryWorkspace.svelte';");
    expect(page).toContain("import ImportSheet from '$lib/components/ImportSheet.svelte';");
    expect(page).toContain('hydrateCachedLibrarySnapshot();');
    expect(page).toContain('void loadStats({ suppressErrors: true });');
    expect(page).toContain('await refreshBookmarks();');
    expect(page).toContain('<LibraryWorkspace onopenimport={() => (showImportSheet = true)} />');
  });

  test('preserves multiple layout modes for bookmark presentation', () => {
    const switcher = read('src/lib/components/LayoutSwitcher.svelte');
    const feed = read('src/lib/components/BookmarkFeed.svelte');

    expect(switcher).toContain("{ id: 'focus', label: 'Focus'");
    expect(switcher).toContain("{ id: 'grid', label: 'Grid'");
    expect(switcher).toContain("{ id: 'list', label: 'List'");
    expect(feed).toContain("layoutMode.value === 'grid'");
    expect(feed).toContain("layoutMode.value === 'list'");
    expect(feed).toContain("toggleFavorite(bookmark.id)");
    expect(feed).toContain("openInBrowser(bookmark.tweet_url)");
    expect(feed).toContain("selectedTag.set(tag)");
  });

  test('keeps keyboard-friendly search controls', () => {
    const searchInput = read('src/lib/components/SearchInput.svelte');

    expect(searchInput).toContain("if (event.key === 'Escape')");
    expect(searchInput).toContain("if (event.key === '/' && !['INPUT', 'TEXTAREA'].includes((event.target as HTMLElement).tagName))");
    expect(searchInput).toContain('placeholder="Search your archive by text, author, or tag"');
    expect(searchInput).toContain('selectedTag.clear()');
  });

  test('keeps navigation, favorites, settings, and import flows discoverable', () => {
    const sidebar = read('src/lib/components/LibrarySidebar.svelte');
    const importSheet = read('src/lib/components/ImportSheet.svelte');

    expect(sidebar).toContain("label: 'Library'");
    expect(sidebar).toContain("label: 'Recent'");
    expect(sidebar).toContain("label: 'Favorites'");
    expect(fileExists('src/routes/settings/+page.svelte')).toBe(true);
    expect(importSheet).toContain('Import directly from X');
    expect(importSheet).toContain('Import a file');
  });
});
