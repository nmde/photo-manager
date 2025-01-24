<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { fileStore } from '../stores/fileStore';
import { Photo, createPhoto } from '../classes/Photo';

const router = useRouter();
const {
  setWorkingDir,
  loadPhotos,
  setFiles,
  generateThumbnails,
  groupRaws,
  removeDeleted,
  setFolderStructure,
} = fileStore;

const loading = ref(false);
const deletedDialog = ref(false);
const deleted = ref<string[]>([]);
const initializing = ref(false);
const initializingProgress = ref(0);
const fileCount = ref(0);
const reading = ref('');

/**
 * Uses dir to quickly read a directory's contents.
 * @param path - The path to read.
 */
async function readDir(path: string, top = true) {
  const { join } = await import('@tauri-apps/api/path');
  const { Command } = await import('@tauri-apps/plugin-shell');
  console.log(`Reading ${path}`);
  reading.value = path;
  let files: string[] = [];
  let dirs: string[] = [];
  const output = await Command.create('cmd', ['/C', 'dir', path]).execute();
  console.log(output.stdout);
  if (output.stderr.length > 0) {
    console.error(output.stderr);
  } else {
    const it = output.stdout.matchAll(/[0-9]{2}\/[0-9]{2}\/[0-9]{4}.*$/gm);
    let curr = it.next();
    while (!curr.done) {
      if (curr.value[0].indexOf('<DIR>') > 0) {
        const dir = curr.value[0].replace(
          /[0-9]{2}\/[0-9]{2}\/[0-9]{4}\s+[0-9]{2}:[0-9]{2}\s[AP]M\s+<DIR>\s+/,
          '',
        );
        if (['.', '..'].indexOf(dir) < 0) {
          dirs.push(await join(path, dir));
        }
      } else {
        files.push(
          await join(
            path,
            curr.value[0].replace(
              /[0-9]{2}\/[0-9]{2}\/[0-9]{4}\s+[0-9]{2}:[0-9]{2}\s[AP]M\s+[0-9,]+\s+/,
              '',
            ),
          ),
        );
      }
      curr = it.next();
    }
    if (top) {
      fileCount.value += dirs.length;
    }
    for (const dir of dirs) {
      const r = await readDir(dir, false);
      files = files.concat(r.files);
      dirs = dirs.concat(r.dirs);
    }
  }
  if (top) {
    initializingProgress.value += 1;
  }
  return { dirs, files };
}

/**
 * Prompts the user to select the folder to manage.
 */
async function openFolder() {
  loading.value = true;
  const { open } = await import('@tauri-apps/plugin-dialog');
  const { convertFileSrc } = await import('@tauri-apps/api/core');
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected && typeof selected === 'string') {
    initializing.value = true;
    await setWorkingDir(selected);
    const files: Record<string, Photo> = {};
    const existing = { ...(await loadPhotos()) };
    let fullFileList: string[] = [];
    let raws: string[] = [];
    let videos: string[] = [];
    console.log('Loaded photos');
    const folder = await readDir(selected);
    setFolderStructure(folder);
    fullFileList = folder.files;
    console.log('Read dir');
    const rawPhotos: Photo[] = [];
    fullFileList.forEach(async (file) => {
      if (existing[file]) {
        files[file] = existing[file];
        delete existing[file];
      } else {
        files[file] = createPhoto(file, convertFileSrc(file));
      }
      if (/^.*\.(ORF|NRW|HEIC|TIFF|TIF)$/.test(file.toUpperCase())) {
        files[file].data.raw = true;
        raws.push(file);
      } else if (/^.*\.(3GP|AVI|MOV|MP4|MTS|WAV|WMV|M4V|WEBM)$/.test(file.toUpperCase())) {
        files[file].data.video = true;
        videos.push(file);
      }
      if (files[file].data.raw) {
        rawPhotos.push(files[file]);
      }
    });
    deleted.value = Object.keys(existing);
    setFiles(files);
    groupRaws(rawPhotos);
    if (raws.length > 0 || videos.length > 0) {
      generateThumbnails(raws, videos);
      if (deleted.value.length > 0) {
        deletedDialog.value = true;
      } else {
        router.push('/tagger');
      }
    } else if (deleted.value.length > 0) {
      deletedDialog.value = true;
    } else {
      router.push('/tagger');
    }
  }
  loading.value = false;
}
</script>

<template>
  <v-main>
    <v-container>
      <v-row>
        <v-col cols="4"></v-col>
        <v-col cols="12">
          <div class="main">
            <h1>Photo Manager</h1>
            <v-btn color="primary" @click="openFolder" :loading="loading">Open Folder</v-btn>
          </div>
        </v-col>
        <v-col cols="4"></v-col>
      </v-row>
    </v-container>
    <v-dialog v-model="deletedDialog" persistent>
      <v-card>
        <v-card-title>Missing Files</v-card-title>
        <v-card-text>
          The following files could not be found:
          <ul>
            <li v-for="(file, i) in deleted" :key="i">{{ file }}</li>
          </ul>
        </v-card-text>
        <v-card-actions>
          <v-btn
            color="primary"
            @click="
              async () => {
                for (let i = 0; i < deleted.length; i += 1) {
                  removeDeleted(deleted[i]);
                }
                router.push('/tagger');
              }
            "
            >Remove Records &amp; Continue</v-btn
          >
          <v-btn color="primary" @click="router.push('/tagger')">Continue Without Removing</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <v-dialog v-model="initializing" persistent>
      <v-card>
        <v-card-title>Initializing</v-card-title>
        <v-card-text>
          <p v-if="reading.length > 0">Reading {{ reading }}</p>
          <v-progress-linear
            :model-value="(initializingProgress / fileCount) * 100"
            color="primary"
          ></v-progress-linear>
        </v-card-text>
      </v-card>
    </v-dialog>
  </v-main>
</template>

<style scoped>
.main {
  text-align: center;
}

.main > h1 {
  margin-bottom: 28px;
}
</style>
