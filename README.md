This is web port of the emulator.

Build and run with
```
wasm-pack build --release --target web
cp index.html pkg/
cp script.js pkg/
cd pkg
python server.py
```
Then go to http://localhost:8080/index.html

Note: If things don't work, try changing port number (ie. 8080 to 8000). I have no idea why it works...

Suggested reading: https://rustwasm.github.io/docs/book/introduction.html
