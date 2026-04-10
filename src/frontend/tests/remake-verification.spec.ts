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
  test('removes the stats route and stale stats navigation affordances', () => {
    expect(fileExists('src/routes/stats/+page.svelte')).toBe(false);

    const header = read('src/lib/components/Header.svelte');
    expect(header).not.toContain('href="/stats"');
    expect(header).toContain('href="/settings"');
  });

  test('keeps the primary library shell wired to preserved workflows', () => {
    const page = read('src/routes/+page.svelte');

    expect(page).toContain("import BookmarkList from '$lib/components/BookmarkList.svelte';");
    expect(page).toContain("import DateFilter from '$lib/components/DateFilter.svelte';");
    expect(page).toContain("import ImportModal from '$lib/components/ImportModal.svelte';");
    expect(page).toContain("import LayoutToggle from '$lib/components/LayoutToggle.svelte';");
    expect(page).toContain("import SearchBar from '$lib/components/SearchBar.svelte';");
    expect(page).toContain("import Sidebar from '$lib/components/Sidebar.svelte';");
    expect(page).toContain('await refreshBookmarks();');
    expect(page).toContain('void loadStats({ suppressErrors: true });');
    expect(page).toContain('<BookmarkList items={bookmarks.value} />');
  });

  test('preserves multiple layout modes for bookmark presentation', () => {
    const toggle = read('src/lib/components/LayoutToggle.svelte');
    const list = read('src/lib/components/BookmarkList.svelte');

    expect(toggle).toContain("{ id: 'default', label: 'Focus' }");
    expect(toggle).toContain("{ id: 'cards', label: 'Grid' }");
    expect(toggle).toContain("{ id: 'compact', label: 'List' }");
    expect(list).toContain("layoutMode.value === 'cards'");
    expect(list).toContain("layoutMode.value === 'compact'");
    expect(list).toContain('<BookmarkCard {bookmark} />');
    expect(list).toContain('<BookmarkRow {bookmark} />');
  });

  test('keeps keyboard-friendly search controls', () => {
    const searchBar = read('src/lib/components/SearchBar.svelte');

    expect(searchBar).toContain("if (e.key === 'Escape')");
    expect(searchBar).toContain("if (e.key === '/' && !['INPUT', 'TEXTAREA'].includes((e.target as HTMLElement).tagName))");
    expect(searchBar).toContain("placeholder=\"Search your archive by text, author, or tag\"");
    expect(searchBar).toContain('selectedTag.clear()');
  });

  test('keeps sidebar navigation, favorites, and import/settings flows discoverable', () => {
    const sidebar = read('src/lib/components/Sidebar.svelte');
    const importModal = read('src/lib/components/ImportModal.svelte');

    expect(sidebar).toContain("label: 'Library'");
    expect(sidebar).toContain("label: 'Recent'");
    expect(sidebar).toContain("label: 'Favorites'");
    expect(fileExists('src/routes/settings/+page.svelte')).toBe(true);
    expect(importModal).toContain('Import directly from X');
    expect(importModal).toContain('Import a file');
  });
});
