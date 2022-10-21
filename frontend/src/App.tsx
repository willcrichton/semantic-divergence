import { useEffect, useRef, useState } from "react";
import "./App.scss";
import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { rust } from "@codemirror/lang-rust";

import {
  interpret,
  set_panic_hook,
} from "../../crates/web-bindings/pkg/web_bindings";

set_panic_hook();

let DEFAULT = `
let a = 1;
let mut b;
b = &a;
let c = *b;`.trim();

let runInterpreter = (contents: string): string => interpret(`{${contents}}`);

function App() {
  let [editor, setEditor] = useState<EditorView | null>(null);
  let [output, setOutput] = useState(runInterpreter(DEFAULT));

  let ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    setEditor(
      new EditorView({
        parent: ref.current!,
        state: EditorState.create({
          doc: DEFAULT,
          extensions: [basicSetup, rust()],
        }),
      })
    );
  }, []);

  return (
    <div className="App">
      <div className="editor">
        <h1>Input</h1>
        <button
          onClick={() => {
            let contents = editor!.state.doc.toJSON().join("\n");
            setOutput(runInterpreter(contents));
          }}
        >
          Run
        </button>
        <div ref={ref} />
      </div>
      <div className="output">
        <h1>Output</h1>
        <pre>{output}</pre>
      </div>
    </div>
  );
}

export default App;
