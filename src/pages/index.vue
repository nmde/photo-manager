<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useFileStore } from '../stores/fileStore';
import { Photo, createPhoto } from '~/classes/Photo';

const router = useRouter();
const { setWorkingDir, loadPhotos, removeDeleted, setFiles } =
  useFileStore();

const loading = ref(false);
const deletedDialog = ref(false);
const deleted = ref<string[]>([]);
const thumbnailDialog = ref(false);
const thumbnailCount = ref(0);
const thumbnailProgress = ref(0);
const initializing = ref(false);
const initializingProgress = ref(0);
const fileCount = ref(0);

function clean(path: string) {
  return path.replace(/[/\\]/g, '-').replace(':', '');
}

/**
 * Prompts the user to select the folder to manage.
 */
async function openFolder() {
  loading.value = true;
  const { open } = await import('@tauri-apps/api/dialog');
  const { readDir, exists, createDir, removeFile } = await import('@tauri-apps/api/fs');
  const { join, appDataDir } = await import('@tauri-apps/api/path');
  const { convertFileSrc } = await import('@tauri-apps/api/tauri');
  const { Command } = await import('@tauri-apps/api/shell');
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected && typeof selected === 'string') {
    console.log(selected);
    initializing.value = true;
    await setWorkingDir(selected);
    const files: Record<string, Photo> = {};
    const existing = { ...(await loadPhotos()) };
    const fullFileList: any[] = [];
    let raws: any[] = [];
    let videos: any[] = [];
    const expandDir = async (dir: string) => {
      const d = await readDir(dir);
      for (const file of d) {
        if (file.children !== undefined) {
          await expandDir(file.path);
        } else {
          fullFileList.push(file);
        }
      }
    }
    await expandDir(selected);
    console.log(fullFileList);
    fileCount.value = fullFileList.length;
    fullFileList.forEach(async (file) => {
      if (existing[file.path]) {
        files[file.path] = existing[file.path];
        delete existing[file.path];
      } else {
        files[file.path] = createPhoto(file.path, file.path);
      }
      initializingProgress.value += 1;
      if (/^.*\.(ORF|NRW)$/.test(file.path.toUpperCase())) {
        raws.push(file);
      } else if (/^.*\.(3GP|AVI|MOV|MP4|MTS|WAV|WMV)$/.test(file.path.toUpperCase())) {
        videos.push(file);
      }
    });
    const dir = await appDataDir();
    if (!(await exists(dir))) {
      await createDir(dir);
    }
    const thumbnailDir = await join(dir, 'thumbnails');
    if (!(await exists(thumbnailDir))) {
      await createDir(thumbnailDir);
    }
    const projectThumbnailDir = await join(
      thumbnailDir,
      selected.replace(/[/\\]/g, '-').replace(':', ''),
    );
    if (!(await exists(projectThumbnailDir))) {
      await createDir(projectThumbnailDir);
    }
    const thumbnails = (await readDir(projectThumbnailDir)).map((p) => p.name);
    deleted.value = Object.keys(existing);
    for (let i = 0; i < deleted.value.length; i += 1) {
      await removeDeleted(deleted.value[i]);
      const thumbnailPath = await join(
        projectThumbnailDir,
        `${deleted.value[i].replace(/\..*$/, '')}.jpg`,
      );
      if (await exists(thumbnailPath)) {
        await removeFile(thumbnailPath);
      }
    }
    if (raws.length > 0 || videos.length > 0) {
      thumbnailDialog.value = true;
      thumbnailProgress.value = 0;
      thumbnailCount.value = raws.length + videos.length;
      for (const raw of raws) {
        const thumbnailFile = `${clean(raw.path as string).replace(/\..*$/, '')}.jpg`;
        const thumbnailPath = await join(projectThumbnailDir, thumbnailFile);
        if (thumbnails.indexOf(thumbnailFile) < 0) {
          const convertOutput = await new Command('magick', [raw.path, thumbnailPath]).execute();
          if (convertOutput.code !== 0) {
            console.error(convertOutput.stderr);
          }
          const resizeOutput = await new Command('magick', [
            thumbnailPath,
            '-resize',
            '800x800',
            thumbnailPath,
          ]).execute();
          if (resizeOutput.code !== 0) {
            console.error(resizeOutput.stderr);
          }
        }
        files[raw.path].data.thumbnail = convertFileSrc(thumbnailPath);
        thumbnailProgress.value += 1;
      }
      for (const video of videos) {
        const thumbnailFile = `${clean(video.path as string).replace(/\..*$/, '')}.png`;
        const thumbnailPath = await join(projectThumbnailDir, thumbnailFile);
        if (thumbnails.indexOf(thumbnailFile) < 0) {
          const convertOutput = await new Command('ffmpeg', [
            '-i',
            video.path,
            '-ss',
            '00:00:01.00',
            '-vframes',
            '1',
            thumbnailPath,
          ]).execute();
          if (convertOutput.code !== 0) {
            console.error(convertOutput.stderr);
          }
        }
        thumbnailProgress.value += 1;
        files[video.path].data.video = true;
        files[video.path].data.thumbnail = convertFileSrc(thumbnailPath);
      }
      setFiles(files);
      thumbnailDialog.value = false;
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
    <v-dialog v-model="deletedDialog">
      <v-card>
        <v-card-title>Missing Files</v-card-title>
        <v-card-text>
          The following files could not be found:
          <ul>
            <li v-for="(file, i) in deleted" :key="i">{{ file }}</li>
          </ul>
        </v-card-text>
        <v-card-actions>
          <v-btn color="primary" @click="router.push('/tagger')">Continue</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <v-dialog v-model="thumbnailDialog" persistent>
      <v-card>
        <v-card-title>Generating Thumbnails</v-card-title>
        <v-card-text>
          Progress: {{ thumbnailProgress }} / {{ thumbnailCount }}
          <v-progress-linear
            :model-value="(thumbnailProgress / thumbnailCount) * 100"
            color="primary"
          ></v-progress-linear>
        </v-card-text>
      </v-card>
    </v-dialog>
    <v-dialog v-model="initializing" persistent>
      <v-card>
        <v-card-title>Initializing</v-card-title>
        <v-card-text>
          Progress: {{ initializingProgress }} / {{ fileCount }}
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
