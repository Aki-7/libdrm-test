# libdrm test

Direct Rendering Manager を 扱ってみる。
Rustの勉強もしたかったので、

https://github.com/dvdhrm/docs/tree/master/drm-howto

↑これをRustで実装。
libdrmのRust実装らしき物もいくつか見受けられたが、あまり使えなさそうだったので、自分でFFIを作ってlibdrmをRustから利用するようにした。

RustのFFI: https://doc.rust-lang.org/nomicon/ffi.html

この辺を触るにはちょっと、Rustはツールがととのってないのかな、という印象をうけました。

最後の方の終了処理は面倒になって手を回してないので、注意してください。何かが壊れたりはしないと思いますが。

## 何ができるか、

libdrmを使ってディスプレイに描画させる。ただそれだけです。

## Setup

- rustのインストール
- git clone
- cargo run

注意: 

Dumb Bufferという便利な機能を使っているため、GPUのグラフィックカード経由でのモニターへの出力はできませんでした。
GPUを使わず、CPUのグラフィックカードを使う必要があるかもしれません。

CUIモードで実行しないと動作しません。

モニターに繋がっていないと動作しません。
