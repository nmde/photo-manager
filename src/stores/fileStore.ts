import type { Sort } from '@/api/photos';

export const useFileStore = defineStore('files', () => {
  const darkMode = ref(true);
  const currentDir = ref('');
  const query = ref<string[]>([]);
  const sortBy = ref<Sort>('name');
  const calendarViewDate = ref(new Date());
  const itemsPerRow = ref(4);
  const lastSetDate = ref(new Date());

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

  return {
    darkMode,
    currentDir,
    query,
    sortBy,
    calendarViewDate,
    itemsPerRow,
    lastSetDate,
    toggleTheme,
    setTheme,
    setCurrentDir,
    setQuery,
    setCalendarViewDate,
    setItemsPerRow,
    setLastDate,
  };
});
