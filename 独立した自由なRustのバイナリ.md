# A Freestandig Rust Binary

独自のオペレーティングシステムカーネルを作成する最初のステップは、標準ライブラリ(std)にリンクしない実行可能ファイルを作成することです。
これにより、Rustで作成したコードをベアメタル上で実行することが可能になります。もちろん、基盤となるオペレーティングシステムなしでね。

このブログの内容は[Github](https://github.com/phil-opp/blog_os)で公開しているから、もし何かしらの問題や質問があったらissueを作ってね。あと、このブログの一番下でコメントもできるよ。完成したソースコードは[このポスト](https://github.com/phil-opp/blog_os/tree/post-01)で見つけられるよ。
それじゃあ始めようか。

## Introduction


## The no_std Attribute
まず僕らは、stdライブラリからの独立して自由な世界を勝ち取らなければならない。stdライブラリはオペレーティングシステム(OS)に依存していて、OSに依存したライブラリで作られたOSは基盤OSに依存してしまうからね。

というわけでmain.rsにno_stdアトリビュートを記述して、コンパイラがstdライブラリをリンクしないようにする。stdの世界よ、さらばだ。

``` rs
//main.rs

#![no_std]

fn main() {
    println!("Hello, world!");
}
```

これでcargo buildすると、println!はstdライブラリのマクロなので、
下記のようにconnot find macro in this scopeのErrorで怒られる。

``` sh
error: cannot find macro `println!` in this scope
 --> src/main.rs:4:5
  |
4 |     println!("Hello, world!");
  |     ^^^^^^^

error: aborting due to previous error

error: Could not compile `iranika_os`.

To learn more, run the command again with --verbose.
```

じゃあ、`println!`の行を削除して、何もしないmainなら動くはずだ！！

``` rs
#![no_std]

fn main() {
}

```

コードをコンパイルしてみる。

``` sh
error: `#[panic_handler]` function required, but not found
error: language item required, but not found: `eh_personality`
error: aborting due to 2 previous errors
error: Could not compile `iranika_os`.
```

何やら`要求されたが、見つからなかった。`というエラーでコンパイルが失敗しましたね。


## panic_handler
panic_handlerというのは、panicが発生したときに呼び出される関数とのこと。
rustcはpanicが発生したときに呼び出す関数がないことを許してくれない。これは、安全な世界を作るためのルールなのだ。
stdには独自のpanic_handlerが定義されていたが、stdに別れを告げたこの世界では僕らが新たに定義する必要がある:

``` rs
use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

`PanicInfo`には、panicが発生したファイルと行、およびオプションのパニックメッセージが含まれている。このpanic_handlerの関数はreturnすべきではない。なので`diverging function`(分岐関数?）として`never`型(=!)を返し続けるように、とりあえず定義する。
今のところは、とりあえず無限ループでもするように実装しておく。
今目指している目的地はstdのない世界で実行形式ファイルを作り出すことだからね。

## eh_personality
`eh_personality`はLanguage itemsの一つです。
Language itemsはコンパイラが内部的に必要とする特殊な機能（および型）のこと。
例えばCopyトレイトは、Language itemのひとつで、どの型がCopyセマンティックを持っているのかコンパイラに明示するための役割を持っています。
以下のCoreライブラリのCopyトレイトの実装を見ると、特別なアトリビュートとして`#[lang = "copy"]`が付与されています。

``` rs
#[stable(feature = "rust1", since = "1.0.0")]
#[lang = "copy"]
pub trait Copy : Clone {
    // Empty.
}
```

じゃあ、`eh_personality`を付与する関数を実装すればいいのか！！と考える。
しかし、`eh_personality`はスタックの巻き戻しを実装するために使用される関数に付与するLanguage itemsで、処理が複雑すぎる。
Rustのデフォルト動作ではpanicの際に巻き戻しを利用して、すべてのライブスタック変数のデストラクタを実行して使用されているメモリが確実に開放する。その後、親スレッドはパニックをキャッチして実行を継続できるようになる。
しかし、巻き戻し処理は複雑な処理であり、大抵はいくつかのOS特有のライブラリ(例えばLinuxではlibunwind, Windowsではstructured exceptin handling)に依存して実装される(Rustもそう)。だが私達はOSを作っているのであって、OSに依存してしまうような`eh_personality`は使いたくない。使いたくないのだ。

そこで、Rust(rustc)はpanicで中止(abort)するオプションを提供している。
これにより、スタックの巻き戻し処理の実装(`eh_personality`)をコンパイラは要求しなくなる。
複数のやり方があるが、簡単な方法は次の行をCargo.tomlに追加することだ。

`Cargo.toml`:
``` toml
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

これでdevプロファイル(cargo build)とreleaseプロファイル(carg build --release)の両方で、panic時にabort(中止)するようになります。
これで、`eh_personality`の実装は必要なくなりました！！(ドヤァ)

鼻高々に`cargo build`しましょう！！

``` sh
error: requires `start` lang_item
error: aborting due to previous error
error: Could not compile `iranika_os`.
```

Opps!!

まだまだ道のりは長そうです。

<!--
確かに独自のLanguage itemsの実装を提供することは可能ですが、それは最後手段です。
なぜなら、Language itemsは非常に不安定な実装がされており、型チェックがされません。
そのため、コンパイラはlanguage itemsの関数が正しい引数を持っているかどうかチェックしていません。
-->

## The start attribute

忌々しいErrorを倒すためには、まずプログラム実行時の前処理について知らなければならない。
焦って`start`lang_itemの実装をしようとしてはいけない。

main関数はプログラムを実行したときに最初に呼び出される関数だと思っていないだろうか(僕も昔はそうだった)。
しかし、ほとんどのプログラミング言語にはランタイムシステムがあり、ガベージコレクション(例：java)やソフトウェアスレッド(例:GoのGoルーチン)などを担っている。このランタイムは、自分自身を初期化する必要があるため、mainの前に呼び出す必要があるのだ。

stdライブラリをリンクする一般的なRustで書かれたバイナリは、crt0(C runtime zero)というCランタイムライブラリで実行が開始され、Cアプリケーションの環境が設定される。これには、スタックの作成と正しいレジスタへの引数の配置が含まれる。
その後、Cランタイムは`start`language itemの付与されたRustランタイムのエントリポイントを呼び出す。これには、スタックオーバーフローガードの設定や、panic時のバックトレースのprintなど、いくつか小さな処理をする。
その後、やっとmain関数が呼び出される。

`C runtime zero`
↓
`Rust runtime`
↓
`fn main()`

だがそれは、stdライブラリをリンクした世界での話なのだ。
stdに別れを告げた僕らのバイナリファイルは、crt0とRustランタイムへのアクセスを持っていない。つまりエントリポイントがどこにも定義されていないのだ。
`start`lang_itemの実装はRustランタイムが呼び出すエントリーポイントの実装でしかない。Rustランタイムはcrt0を必要とするため、crt0へのアクセスを持たない僕らは、`start`lang_itemを実装したところで呼び出せないのだ。

じゃあ、crt0のエントリポイントを直接上書きすればいいのでは？
ということで、次の話につながる。


## Overwriting the Entry Point

そもそもcrt0やRustランタイムのエントリポイントチェーンを作っているのはコンパイラだ。
rustc(Rust compiler)は通常、先の通りにランタイムへのエントリポイントのチェーンを作る。
というわけで、rustcに通常のエントリポイントチェーンを使用したくないことをRustコンパイラに伝えるため、　`#![no_main]`属性を追加する。

``` rs
#![no_std]
#![no_main]  //new

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

```

ここで僕らはとんでもないことをしたことに気づくかもしれない。
だって、mainはそれを呼び出すランタイムがないとうまく機能しないから意味がないのです。
代わりにOSが呼び出すエントリポイントを上書きしてはどうだろうか。
()

エントリポイントの規則はOSによって異なるが、僕らの作るカーネルは以下のLinuxを使うことにする。
Linuxという先人は偉大だ。

### Linux

Linuxではデフォルトのエントリポイントは`_start`。
リンカは単にその名前の関数を探して、実行可能ファイルのエントリポイントとして設定します。
そのため、Linuxではエントリポイントを上書きする際に独自の_start関数を定義します。

add `main.rs`:
``` rs
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
```

<!-- no trance 
no_mangle属性による名前マングリングを無効にすることが重要です。
そうしないと、コンパイラは、リンカが認識できないような不可解な_ZN3blog_os4_start7hb173fedf945531caEシンボルを生成します。
また、関数にextern "C"のマークを付けて、（指定されていないRust呼び出し規則の代わりに）この関数にC呼び出し規則を使用するようコンパイラーに指示する必要があります。
-->

戻り値の`never`型(= !)は、`diverging function`であり、
戻ることができないことを意味してる。
実行ファイルのエントリポイントはどの関数からも呼び出されず、オペレーティングシステムまたはブートローダーから直接呼び出されるため、戻ることがないのは必須だ。
戻る代わりに、OSの`exit`システムコールを呼び出す。
僕らの場合は、独立した実行可能ファイルが帰ってきたらやるべきことはなにもないので、マシンをシャットダウンすることは合理的な動作かもしれない。
まぁそんなOSを作ろうとしていないから、とりあえず無限ループすることにしておく。

ちなみに、今すぐ`_start`を追加したコードをビルドしようとすると、見苦しいリンカエラーが発生する。

```sh
error: linking with `cc` failed: exit code: 1
  |
  = note: "cc" "-m64" "-L" "/Users/kubota/.rustup/toolchains/stable-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "/Users/kubota/code/iranika_os/target/debug/deps/iranika_os-6a4cf7f43d962fa1.54dqdp6cja0w2c9r.rcgu.o" "-o" "/Users/kubota/code/iranika_os/target/debug/deps/iranika_os-6a4cf7f43d962fa1" "-Wl,-dead_strip" "-nodefaultlibs" "-L" "/Users/kubota/code/iranika_os/target/debug/deps" "-L" "/Users/kubota/.rustup/toolchains/stable-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib" "/Users/kubota/.rustup/toolchains/stable-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcore-b6f6b59b6a29ec3a.rlib" "/Users/kubota/.rustup/toolchains/stable-x86_64-apple-darwin/lib/rustlib/x86_64-apple-darwin/lib/libcompiler_builtins-f31526d6d86acb04.rlib"
  = note: ld: entry point (_main) undefined. for architecture x86_64
          clang: error: linker command failed with exit code 1 (use -v to see invocation)

error: aborting due to previous error
error: Could not compile `iranika_os`.

```

この問題はrustcがCランタイムの起動ルーチンをまだリンクしているから起こる。
CランタイムはC標準ライブラリのlibcのいくつかのシンボルを要求するが、それらはno_stdアトリビュートによって消し去ったのでリンクされていない。
そこで、Cの起動ルーチンのリンクを取り除く。
これを行うには、nostartfilesフラグをリンカに渡す。

cargoを介してリンカに渡す一つの方法は、cargo rustcコマンド。
このコマンドは、cargo buildと同じように動作するが、rustcにオプションを渡すことができる。
rustcには引数をリンカに渡す`-Z pre-link-arg`フラグがあるので、cargo->rustc->リンカ、という感じでnostartfilesフラグが渡せる。
コマンドにするとこう。

``` sh
> cargo rustc -- -Z pre-link-arg=-nostartfiles
```

-Zフラグは不安定なため、このコマンドはnightly Rustでのみ使えることに注意してください。これで僕らは真の自由を手に入れたバイナリファイルをビルドすることができるようになりました。

### mac
```
#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {}
}
```

To build it and link `libSystem`, we execute:

```sh
cargo rustc -- -Z pre-link-arg=-lSystem
```

## Summary

ちなみにrustc 1.30以降の場合は、panic_handlerがpanic_implementationと仕様変更されていたりした。
まぁrustcさんがコンパイルエラーのヘルプで教えてくれたので、どうにかなったが。
以下は1.30 nightlyのコード(mac版)
エントリポイントの関数は環境に合わせて変更してください。

``` rs
#![no_std]
#![no_main]
#![feature(panic_implementation)]

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_implementation]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {}
}
```