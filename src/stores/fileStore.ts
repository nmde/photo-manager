import type { Sort } from '@/api/app';
import type { ThemeSetting } from '@/api/settings';

export const useFileStore = defineStore('files', () => {
  const theme = ref<ThemeSetting>('Dark');
  const query = ref<string[]>([]);
  const sortBy = ref<Sort>('name');
  const calendarViewDate = ref(new Date());
  const itemsPerRow = ref(4);
  const lastSetDate = ref(new Date());
  const globalError = ref<string>();
  const searchHistory = ref<string[][]>([]);

  function toggleTheme() {
    theme.value = theme.value === 'Dark' ? 'Light' : 'Dark';
  }

  function setTheme(value: ThemeSetting) {
    theme.value = value;
  }

  function setQuery(q: string[], s: Sort) {
    query.value = q;
    sortBy.value = s;
  }

  function setCalendarViewDate(date: Date) {
    calendarViewDate.value = date;
  }

  function setItemsPerRow(value: number) {
    if (value > 0) {
      itemsPerRow.value = value;
    }
  }

  function setLastDate(date: Date) {
    lastSetDate.value = date;
  }

  function reportError(message: string) {
    globalError.value = message;
  }

  function pushHistory(q: string[]) {
    const existing = searchHistory.value.findIndex(
      e => e.length === q.length && e.every((v, i) => v === q[i]),
    );
    if (existing !== -1) {
      searchHistory.value.splice(existing, 1);
    }
    searchHistory.value.unshift([...q]);
    if (searchHistory.value.length > 5) {
      searchHistory.value.pop();
    }
  }

  return {
    theme,
    query,
    sortBy,
    calendarViewDate,
    itemsPerRow,
    lastSetDate,
    globalError,
    toggleTheme,
    setTheme,
    setQuery,
    setCalendarViewDate,
    setItemsPerRow,
    setLastDate,
    reportError,
    searchHistory,
    pushHistory,
  };
});
