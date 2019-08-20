# ruxv6

Goal: reimplement [xv6 OS](https://github.com/mit-pdos/xv6-public) in Rust.

## Requirements

- Nightly [Rust](https://www.rust-lang.org/tools/install)
    - run `$ rustc default nightly`.
- [cargo](https://github.com/rust-lang/cargo)
- [xargo](https://github.com/japaric/xargo)
    - run `$ cargo install xargo`.
- (QEMU)

## Execution

**Currently, first you have to build [xv6-public](https://github.com/mit-pdos/xv6-public), then copy `kernel` and `fs.img` to the root of this project.**

<details>
<summary>example</summary>

```
$ cd ../xv6-public
$ make kernel fs.img
$ cd ../ruxv6
$ cp kernel fs.img .
```

</details>

---

You can try this OS on QEMU by following command.

```
$ make qemu
```