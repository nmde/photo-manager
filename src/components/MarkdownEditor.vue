<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { fileStore } from '../stores/fileStore';

const { theme, wikiPages, people, places } = fileStore;

const props = defineProps<{
  text: string;
}>();

const emit = defineEmits<{
  (e: 'save', content: string): void;
}>();

type TextBlock = {
  text: string;
  style: string;
  original: string;
  symbol: string;
  to?: string;
};

const textBlocks = ref<TextBlock[]>([]);
const currentEditor = ref();
const cursorLeft = ref(0);
const cursorTop = ref(2);
const startingLink = ref(false);
const linkIndex = ref(-1);
const linkText = ref('');

const linkNames = computed(() => {
  return Object.values(wikiPages)
    .map((page) => `/wiki/${page.data.name}`.replace(/\/\//g, '/'))
    .concat(Object.values(people).map((p) => `/people/${p.data.name}`))
    .concat(Object.values(places).map((p) => `/locations/${p.data.name}`))
    .filter((name) => name.toLowerCase().includes(linkText.value.toLowerCase()));
});

const widthContainer = ref();
function width(text: string, size = '1rem', style = '') {
  const context = (widthContainer.value as HTMLCanvasElement).getContext('2d');
  if (context) {
    context.font = `${size} ${style} "Roboto", sans-serif`;
    return context.measureText(text).width;
  }
  return 0;
}

function getName(path: string) {
  const split = path.split('/');
  return split[split.length - 1];
}

/**
 * Processes text within an HTML element.
 * @param text - The text to process.
 */
function processInnerText(text: string) {
  let blocks: TextBlock[] = [];
  const nextSymbol = /[^\\](\*\*|\*|_|\n|\[\[|!|##)/.exec(text);
  if (nextSymbol == null) {
    blocks.push({
      text,
      style: 'span',
      original: text,
      symbol: '',
    });
  } else {
    const ns = nextSymbol[0].substring(1);
    if (ns === '\n') {
      if (nextSymbol.index + 1 > 0) {
        blocks.push({
          text: text.substring(0, nextSymbol.index + 1),
          style: 'span',
          original: text.substring(0, nextSymbol.index + 1),
          symbol: '',
        });
      }
      blocks.push({
        text: '',
        style: 'br',
        original: '\n',
        symbol: '\n',
      });
      blocks = blocks.concat(processInnerText(text.substring(nextSymbol.index + 1)));
    } else {
      let closeSymbol = text.indexOf(nextSymbol[0].substring(1), nextSymbol.index + nextSymbol[0].length);
      if (ns === '[[') {
        closeSymbol = text.indexOf(']]', nextSymbol.index + nextSymbol[0].length);
      } else if (ns === '##') {
        closeSymbol = text.indexOf('\n', nextSymbol.index + nextSymbol[0].length);
      }
      if (nextSymbol.index > 0 && closeSymbol > 0) {
        blocks.push({
          text: text.substring(0, nextSymbol.index + 1),
          style: 'span',
          original: text.substring(0, nextSymbol.index + 1),
          symbol: '',
        });
      }
      if (nextSymbol != null && closeSymbol < 0) {
        blocks.push({
          text,
          style: 'span',
          original: text,
          symbol: '',
        });
      } else {
        let style = 'i';
        let symbol = nextSymbol[0];
        let renderedText = text.substring(nextSymbol.index + nextSymbol[0].length, closeSymbol);
        switch (ns) {
          case '**':
            style = 'b';
            break;
          case '_':
            style = 'u';
            break;
          case '!':
            style = 'Date';
            break;
          case '##':
            style = 'h2';
            closeSymbol -= 1;
            break;
          case '[[':
            style = 'RouterLink';
            symbol = `${renderedText.replace(getName(renderedText), '')}`;
            renderedText = getName(renderedText);
            break;
          default:
        }
        blocks.push({
          text: renderedText,
          style,
          original: text.substring(nextSymbol.index + 1, closeSymbol + nextSymbol[0].length),
          symbol,
        });
        blocks = blocks.concat(
          processInnerText(text.substring(closeSymbol + nextSymbol[0].length - 1)),
        );
      }
    }
  }
  return blocks;
}

function moveCursor() {
  const cursorPos = (currentEditor.value as HTMLTextAreaElement).selectionStart;
  let renderedText: string[] = [''];
  let offset = 0;
  let originalText: string[] = [''];
  let specialChars: string[] = [''];
  let line = 0;
  let reachedCursor = false;
  let b = 0;
  while (!reachedCursor && b < textBlocks.value.length) {
    const block = textBlocks.value[b];
    originalText[line] += block.original;
    renderedText[line] += block.text;
    if (originalText[line].length > cursorPos - offset - 1) {
      reachedCursor = true;
      if (block.style === 'RouterLink') {
        specialChars[line] += '[[' + block.symbol;
      } else {
        specialChars[line] += block.symbol;
      }
    } else {
      if (block.style === 'RouterLink') {
        specialChars[line] += '[[' + block.symbol + ']]';
      } else {
        specialChars[line] += `${block.symbol}${block.symbol}`;
      }
    }
    if (block.style === 'br' || block.style === 'h2') {
      offset += originalText[line].length;
      line += 1;
      renderedText.push('');
      specialChars.push('');
      originalText.push('');
    }
    b += 1;
  }
  renderedText[line] = renderedText[line].substring(
    0,
    cursorPos - offset - specialChars[line].length,
  );
  cursorLeft.value = width(renderedText[line]);
  cursorTop.value = 2 + 16 * 1.5 * line;
}

function syncContent() {
  textBlocks.value = processInnerText(props.text);
  currentEditor.value.value = props.text;
  currentEditor.value.focus();
}

watch(props, () => {
  syncContent();
});

onMounted(() => {
  syncContent();
});

let timer: NodeJS.Timeout;
function handleKeypress() {
  clearTimeout(timer);
  timer = setTimeout(() => {
    emit('save', currentEditor.value.value);
  }, 500);
}
</script>

<template>
  <div class="markdown-view">
    <textarea
      ref="currentEditor"
      class="current-editor"
      @keyup="
        (e) => {
          const textarea = e.target as HTMLTextAreaElement;
          textBlocks = processInnerText(textarea.value);
          moveCursor();
          if (
            textarea.value[textarea.selectionStart - 1] === '[' &&
            textarea.value[textarea.selectionStart - 2] === '['
          ) {
            startingLink = true;
            linkIndex = textarea.selectionStart - 1;
          } else if (
            (textarea.value[textarea.selectionStart - 1] === '[' &&
              textarea.value[textarea.selectionStart - 2] !== '[') ||
            textarea.value[textarea.selectionStart - 1] === ']'
          ) {
            startingLink = false;
          } else if (startingLink) {
            linkText = textarea.value.substring(linkIndex + 1, textarea.selectionStart);
          }
          handleKeypress();
        }
      "
    ></textarea>
    <div
      class="written"
      :style="{ background: theme ? 'rgb(18,18,18)' : 'white' }"
      @click="
        (e) => {
          const target = e.target as HTMLSpanElement;
          currentEditor.focus();
          const row = Math.floor(e.offsetY / (16 * 1.5));
          let r = 0;
          let b = 0;
          let offset = 0;
          while (b < textBlocks.length && r < row) {
            const block = textBlocks[b];
            offset += block.original.length;
            if (block.style === 'br' || block.style === 'h2') {
              r += 1;
            }
            b += 1;
          }
          currentEditor.selectionEnd =
            offset +
            Math.round(
              target.innerHTML.length *
                (Math.round((e.offsetX / width(target.innerHTML)) * 100) / 100),
            );
          currentEditor.selectionStart = currentEditor.selectionEnd;
          moveCursor();
        }
      "
    >
      <template v-for="(block, idx) in textBlocks" :key="idx">
        <router-link v-if="block.style === 'Date'" :to="`/journal/${block.text}`">{{ block.text }}</router-link>
        <component v-else-if="block.style !== 'RouterLink'" :is="block.style">{{
          block.text
        }}</component>
        <router-link v-else :to="block.original.substring(2, block.original.length - 3)">{{ block.text }}</router-link>
      </template>
      <div
        id="fake-cursor"
        :style="{
          left: `${cursorLeft}px`,
          top: `${cursorTop}px`,
          background: theme ? 'white' : 'black',
        }"
      ></div>
      <v-card
        id="link-suggestions"
        v-if="startingLink"
        :style="{
          left: `${cursorLeft}px`,
          top: `${cursorTop + 18}px`,
        }"
      >
        <v-list>
          <v-list-item
            v-for="page in linkNames"
            :key="page"
            @click="
              () => {
                const cursorPos = currentEditor.selectionStart;
                const beforeCursor = currentEditor.value.substring(0, linkIndex + 1);
                const afterCursor = currentEditor.value.substring(cursorPos + 1);
                currentEditor.value = `${beforeCursor}${page}]]${afterCursor}`;
                currentEditor.selectionStart = cursorPos + page.length + 4;
                currentEditor.selectionEnd = currentEditor.selectionStart;
                currentEditor.focus();
                textBlocks = processInnerText(currentEditor.value);
                startingLink = false;
              }
            "
            >{{ page }}</v-list-item
          >
        </v-list>
      </v-card>
    </div>
  </div>
  <canvas ref="widthContainer"></canvas>
</template>

<style scoped>
.markdown-view {
  width: 100%;
  height: 100%;
  position: relative;
}

.current-editor {
  height: auto;
  width: 100%;
  top: 400px;
  opacity: 1;
  left: 0;
}

#fake-cursor {
  width: 2px;
  height: 18px;
  position: absolute;
  top: 6px;
  transition: all 50ms ease;
}

.written {
  z-index: 1;
  width: fit-content;
  position: absolute;
  max-height: 830px;
  overflow-y: scroll;
  height: 100%;
  width: 100%;
}

#link-suggestions {
  position: absolute;
  width: max-content;
}
</style>
