<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as monaco from 'monaco-editor';

  let {
    value = $bindable(''),
    language = 'plaintext',
    readonly = false,
    height = '300px',
    onchange,
  }: {
    value: string;
    language?: string;
    readonly?: boolean;
    height?: string;
    onchange?: (v: string) => void;
  } = $props();

  let container: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor | null = null;
  let ignoreChange = false;

  onMount(() => {
    editor = monaco.editor.create(container, {
      value,
      language,
      theme: 'vs-dark',
      readOnly: readonly,
      minimap: { enabled: false },
      automaticLayout: true,
      scrollBeyondLastLine: false,
      fontSize: 13,
      lineNumbers: 'on',
      wordWrap: 'on',
    });

    editor.onDidChangeModelContent(() => {
      if (ignoreChange) return;
      const newVal = editor!.getValue();
      value = newVal;
      onchange?.(newVal);
    });
  });

  onDestroy(() => {
    editor?.dispose();
  });

  // Sync external value changes into the editor
  $effect(() => {
    if (editor && editor.getValue() !== value) {
      ignoreChange = true;
      editor.setValue(value);
      ignoreChange = false;
    }
  });

  // Sync language changes
  $effect(() => {
    if (editor) {
      const model = editor.getModel();
      if (model) {
        monaco.editor.setModelLanguage(model, language);
      }
    }
  });

  // Sync readonly changes
  $effect(() => {
    editor?.updateOptions({ readOnly: readonly });
  });
</script>

<div bind:this={container} style="height: {height}; width: 100%; border-radius: 4px; overflow: hidden;"></div>
