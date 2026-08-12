import { TypstEditor } from "./typstEditor";

export interface EditorState {
  lineNumber: number;
  column: number;
  scrollTop: number;
}

export class EditorStateManager {
  private savedEditorState: EditorState | null = null;
  private savedReadingScrollRatio: number = 0;

  saveEditorState(editor: TypstEditor | null): void {
    if (editor) {
      const state = editor.getEditorState();
      if (state) {
        this.savedEditorState = state;
      }
    }
  }

  saveReadingScrollTop(contentEl: HTMLElement | null): void {
    if (contentEl) {
      const { scrollTop, scrollHeight, clientHeight } = contentEl;
      const maxScroll = scrollHeight - clientHeight;
      this.savedReadingScrollRatio = maxScroll > 0 ? scrollTop / maxScroll : 0;
    }
  }

  restoreEditorState(editor: TypstEditor | null): void {
    if (this.savedEditorState && editor) {
      requestAnimationFrame(() => {
        if (editor && this.savedEditorState) {
          editor.restoreEditorState(this.savedEditorState);
        }
      });
    } else if (editor) {
      setTimeout(() => {
        editor?.focus();
      }, 0);
    }
  }

  restoreReadingScrollTop(contentEl: HTMLElement | null): void {
    if (this.savedReadingScrollRatio > 0 && contentEl) {
      setTimeout(() => {
        if (contentEl) {
          const maxScroll = contentEl.scrollHeight - contentEl.clientHeight;
          if (maxScroll > 0) {
            contentEl.scrollTop = this.savedReadingScrollRatio * maxScroll;
          }
        }
      }, 0);
    }
  }

  getSavedReadingScrollRatio(): number {
    return this.savedReadingScrollRatio;
  }

  clear(): void {
    this.savedEditorState = null;
    this.savedReadingScrollRatio = 0;
  }
}
