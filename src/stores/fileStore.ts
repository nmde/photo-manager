import type { Sort } from '@/api/photos';

export const useFileStore = defineStore('files', () => {
  const darkMode = ref(true);
  const currentDir = ref('');
  const query = ref<string[]>([]);
  const sortBy = ref<Sort>('name');
  const calendarViewDate = ref(new Date());
  const itemsPerRow = ref(4);
  const lastSetDate = ref(new Date());
  const globalError = ref<string>();

  function toggleTheme() {
    darkMode.value = !darkMode.value;
  }

  function setTheme(theme: boolean) {
    darkMode.value = theme;
  }

  function setCurrentDir(value: string) {
    currentDir.value = value;
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

  return {
    darkMode,
    currentDir,
    query,
    sortBy,
    calendarViewDate,
    itemsPerRow,
    lastSetDate,
    globalError,
    toggleTheme,
    setTheme,
    setCurrentDir,
    setQuery,
    setCalendarViewDate,
    setItemsPerRow,
    setLastDate,
    reportError,
  };
});
