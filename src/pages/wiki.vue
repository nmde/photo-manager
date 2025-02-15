<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import DecryptionDialog from '../components/DecryptionDialog.vue';
import MarkdownEditor from '../components/MarkdownEditor.vue';
import WikiFolder, { WikiItem } from '../components/WikiFolder.vue';
import { WikiPage } from '../classes/WikiPage';
import { fileStore } from '../stores/fileStore';

const route = useRoute();
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
  localPages.value.forEach((page) => {
    if (page.data.name.indexOf('/') >= 0) {
      let n = page.data.name;
      if (n[0] === '/') {
        n = n.substring(1);
      }
      let base = items;
      const split = n.split('/');
      split.forEach((pathItem, p) => {
        if (!base[pathItem]) {
          base[pathItem] = {
            name: pathItem,
            path: page.data.name,
            folders: {},
            files: {},
            id: pathItem,
          };
        }
        if (p === split.length - 1) {
          base = base[pathItem].files;
        } else {
          base = base[pathItem].folders;
        }
      });
    } else {
      items[page.data.name] = {
        name: page.data.name,
        path: page.data.name,
        folders: {},
        files: {},
        id: page.Id,
      };
    }
  });
  return Object.values(items).sort((a, b) => {
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
  return [split.slice(1, split.length - 1), [split[split.length - 1]]];
}

function join(...fragments: string[]) {
  return fragments.map((f) => f.replace(/\//g, '')).join('/');
}

watch(route, () => {
  const page = route.params.page as string[];
  const match = Object.values(wikiPages).find((p) => p.data.name.replace(/^\//, '') === page.join('/'));
  if (match) {
    activePage.value = match;
    activePagePath.value = page;
  }
  if (page.length > 1) {
    focusedFolder.value = page.slice(0, page.length - 1).join('/');
  } else {
    focusedFolder.value = '';
  }
  initializeStructure();
});

fileStore.on('updateWiki', () => {
  initializeStructure();
});

fileStore.on('decrypted', () => {
  encryptionBlock.value = false;
  if (activePage.value) {
    activePage.value.data.content = wikiPages[activePagePath.value.join('/')].data.content;
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
        <v-spacer></v-spacer>
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
        <v-btn icon @click="() => {
          newFolderName = focusedFolder;
          createFolderDialog = true;
        }">
          <v-icon>mdi-folder</v-icon>
        </v-btn>
      </v-toolbar>
      <v-list>
        <WikiFolder v-for="page in wikiStructure" :key="page.name" :page="page"></WikiFolder>
      </v-list>
    </v-navigation-drawer>
    <div v-if="encryptionBlock">
      Wiki pages are encrypted!
      <br />
      <v-btn color="primary" @click="decryptDialog = true">Decrypt Wiki</v-btn>
    </div>
    <div v-if="activePage != null && !encryptionBlock">
      <h1 v-if="!editTitle" @click="editTitle = true">
        {{ splitNameAndPath(activePage.data.name)[1][0] }}
      </h1>
      <v-text-field
        v-else
        v-model="activePage.data.name"
        @update:focused="
          async (focused) => {
            if (!focused) {
              editTitle = false;
              await setWikiPageTitle(activePage?.Id as string, activePage?.data.name as string);
            }
          }
        "
      ></v-text-field>
      <MarkdownEditor
        :text="activePage.data.content"
        @save="
          async (content) => {
            setWikiPageText(activePage?.Id as string, content);
          }
        "
      ></MarkdownEditor>
    </div>
    <v-dialog v-model="createFolderDialog">
      <v-card>
        <v-card-title>Create Folder</v-card-title>
        <v-card-text>
          <v-text-field label="Folder Name" v-model="newFolderName"></v-text-field>
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
            >Create</v-btn
          >
        </v-card-actions>
      </v-card>
    </v-dialog>
    <DecryptionDialog :value="decryptDialog"></DecryptionDialog>
  </v-main>
</template>

<style scoped>
.wiki-page {
  margin: 8px;
}
</style>
