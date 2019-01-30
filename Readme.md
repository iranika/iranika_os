# 「Writeing an OS in Rust(Second Edtion)」
「[Writeing an OS in Rust(Second Edtion)](https://os.phil-opp.com/)」のblog_osの写経でrustcの内部処理とOS自作について理解を深めることを目的にしています。

上記のブログは英語ですが、稚拙ながら意訳メモも書いていきます。  
~~途中で挫折するかもしれません~~

## 開発環境とバージョン

OS: macOS Mojave
qemu: QEMU emulator version 3.1.0

### Rust関連

rustc 1.33.0-nightly (03acbd71c 2019-01-14)

cargo 1.33.0-nightly (2b4a5f1f0 2019-01-12)

rustup 1.16.0 (beab5ac2b 2018-12-06)  

