<script setup lang="ts">
  import type { WikiPage } from '../../classes/WikiPage';
  import type { WikiItem } from '@/types/WikiItem';
  import { computed, onMounted, ref, watch } from 'vue';
  import { useRoute } from 'vue-router';
  import { fileStore } from '../../stores/fileStore';

  const route = useRoute('/wiki/[...page]');
  const { wikiPages, createWikiPage, setWikiPageText, encrypted, setWikiPageTitle } = fileStore;

  const editTitle = ref(false);
  const localPages = ref<WikiPage[]>([]);
  const activePage = ref<WikiPage | null>(null);
  const activePagePath = ref<string[]>([]);
  const createFolderDialog = ref(false);
  const newFolderName = ref('');
  const focusedFolder = ref('');
  const encryptionBlock = ref(false);
  const decryptDialog = ref(false);

  const wikiStructure = computed(() => {
    const items: Record<string, WikiItem> = {};
    for (const page of localPages.value) {
      if (page.displayName.includes('/')) {
        let n = page.displayName;
        if (n[0] === '/') {
          n = n.slice(1);
        }
        let base = items;
        const split = n.split('/');
        for (const [p, pathItem] of split.entries()) {
          if (!base[pathItem]) {
            base[pathItem] = {
              name: pathItem,
              path: page.displayName,
              folders: {},
              files: {},
              id: pathItem,
            };
          }
          base = p === split.length - 1 ? base[pathItem].files : base[pathItem].folders;
        }
      } else {
        items[page.displayName] = {
          name: page.displayName,
          path: page.displayName,
          folders: {},
          files: {},
          id: page.id,
        };
      }
    }
    return Object.values(items).toSorted((a, b) => {
      const aHasChildren = Object.values(a.folders).length > 0 || Object.values(a.files).length > 0;
      const bHasChildren = Object.values(b.folders).length > 0 || Object.values(b.files).length > 0;
      if (aHasChildren && !bHasChildren) {
        return -1;
      }
      if (bHasChildren && !aHasChildren) {
        return 1;
      }
      return a.name.localeCompare(b.name);
    });
  });

  function initializeStructure() {
    localPages.value = Object.values(wikiPages);
  }

  function splitNameAndPath(path: string) {
    const split = path.split('/');
    return [split.slice(1, -1), [split.at(-1)]];
  }

  watch(route, () => {
    console.log(route.params.page);
    /*
  const match = Object.values(wikiPages).find(
    (p) => p.data.name.replace(/^\//, '') === route.params.page.join('/'),
  );
  if (match) {
    activePage.value = match;
    activePagePath.value = route.params.page;
  }
  if (route.params.page.length > 1) {
    focusedFolder.value = route.params.page.slice(0, route.params.page.length - 1).join('/');
  } else {
    focusedFolder.value = '';
  }
    */
    initializeStructure();
  });

  fileStore.on('updateWiki', () => {
    initializeStructure();
  });

  fileStore.on('decrypted', () => {
    encryptionBlock.value = false;
    if (activePage.value) {
      activePage.value.displayContent = wikiPages[activePagePath.value.join('/')]?.displayContent ?? '';
    }
  });

  onMounted(() => {
    initializeStructure();
    if (encrypted) {
      encryptionBlock.value = true;
    }
  });
</script>

<template>
  <v-main class="wiki-page">
    <v-navigation-drawer permanent>
      <v-toolbar>
        <v-spacer />
        <v-btn
          icon
          @click="
            async () => {
              createWikiPage(focusedFolder);
            }
          "
        >
          <v-icon>mdi-plus</v-icon>
        </v-btn>
        <v-btn
          icon
          @click="
            () => {
              newFolderName = focusedFolder;
              createFolderDialog = true;
            }
          "
        >
          <v-icon>mdi-folder</v-icon>
        </v-btn>
      </v-toolbar>
      <v-list>
        <WikiFolder v-for="page in wikiStructure" :key="page.name" :page="page" />
      </v-list>
    </v-navigation-drawer>
    <div v-if="encryptionBlock">
      Wiki pages are encrypted!
      <br>
      <v-btn color="primary" @click="decryptDialog = true">Decrypt Wiki</v-btn>
    </div>
    <div v-if="activePage != null && !encryptionBlock">
      <h1 v-if="!editTitle" @click="editTitle = true">
        {{ splitNameAndPath(activePage.displayName)[1]?.[0] }}
      </h1>
      <v-text-field
        v-else
        v-model="activePage.displayName"
        @update:focused="
          async focused => {
            if (!focused) {
              editTitle = false;
              await setWikiPageTitle(activePage?.id as string, activePage?.displayName as string);
            }
          }
        "
      />
      <MarkdownEditor
        :text="activePage.displayContent"
        @save="
          async content => {
            setWikiPageText(activePage?.id as string, content);
          }
        "
      />
    </div>
    <v-dialog v-model="createFolderDialog">
      <v-card>
        <v-card-title>Create Folder</v-card-title>
        <v-card-text>
          <v-text-field v-model="newFolderName" label="Folder Name" />
        </v-card-text>
        <v-card-actions>
          <v-btn @click="createFolderDialog = false">Cancel</v-btn>
          <v-btn
            color="primary"
            @click="
              async () => {
                focusedFolder = newFolderName;
                newFolderName = '';
                await createWikiPage(focusedFolder);
                createFolderDialog = false;
              }
            "
          >
            Create
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <DecryptionDialog :value="decryptDialog" />
  </v-main>
</template>

<style scoped>
  .wiki-page {
    margin: 8px;
  }
</style>
