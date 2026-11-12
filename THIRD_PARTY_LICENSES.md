# Third-party licenses

This project is MIT licensed (see [LICENSE](LICENSE)). It also distributes
material from the following projects.

## ALVR

`builds/<version>/ALVR Dashboard.exe` and `ALVR Launcher.exe` are localized
builds of [ALVR](https://github.com/alvr-org/ALVR). They are distributed under
ALVR's MIT license, reproduced here:

```
Copyright (c) 2018-2019 polygraphene
Copyright (c) 2020-2024 alvr-org

Permission is hereby granted, free of charge, to any person
obtaining a copy of this software and associated documentation
files (the "Software"), to deal in the Software without
restriction, including without limitation the rights to use,
copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the
Software is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.
```

## Fonts

`assets/ui-*.ttf` and the fonts inside `languages/*.ttf` are subsets of
**Noto Sans SC** and **Noto Sans KR**, distributed under the
[SIL Open Font License 1.1](https://scripts.sil.org/OFL).
The full license text is included in the release archive as
`licenses/FONT-LICENSE.txt`.

## Rust crates

The tool is built on [`eframe`/`egui`](https://github.com/emilk/egui) (MIT/Apache-2.0),
[`rfd`](https://github.com/PolyMeilex/rfd) (MIT) and
[`serde_json`](https://github.com/serde-rs/json) (MIT/Apache-2.0).
